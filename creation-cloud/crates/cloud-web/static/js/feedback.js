// 官网反馈页只在当前会话有效时启用真实提交，不持久化任何表单内容。
const feedbackPage = document.querySelector("[data-feedback-page]");

if (feedbackPage instanceof HTMLElement) {
  const form = feedbackPage.querySelector("[data-feedback-form]");
  const authState = feedbackPage.querySelector("[data-feedback-auth-state]");
  const login = feedbackPage.querySelector("[data-feedback-login]");
  const result = feedbackPage.querySelector("[data-feedback-result]");
  const submit = feedbackPage.querySelector("[data-feedback-submit]");
  const isEnglish = feedbackPage.dataset.locale === "en";
  let csrfToken = "";

  const labels = isEnglish
    ? {
        ready: "Signed in. Website submission is available.",
        signedOut: "Sign in to Creation Cloud to use the website channel.",
        checkingFailed: "The session could not be checked. Try again shortly or use GitHub Issues.",
        submitting: "Submitting the website ticket…",
        success: "Website ticket submitted. Reference:",
        invalid: "Review the form and confirm that all text is safely redacted.",
        unauthorized: "Your session expired. Sign in again before submitting.",
        limited: "The submission limit has been reached. Please try again later.",
        failed: "The ticket could not be submitted. Try again shortly or use GitHub Issues.",
      }
    : {
        ready: "已登录，可以使用官网渠道。",
        signedOut: "请先登录 Creation Cloud，再使用官网渠道。",
        checkingFailed: "暂时无法检查会话，请稍后重试或使用 GitHub Issues。",
        submitting: "正在提交官网工单…",
        success: "官网工单已提交，编号：",
        invalid: "请检查表单，并确认所有文字已经脱敏。",
        unauthorized: "登录状态已失效，请重新登录后再提交。",
        limited: "已达到提交频率上限，请稍后再试。",
        failed: "工单暂时无法提交，请稍后重试或使用 GitHub Issues。",
      };

  function setFormEnabled(enabled) {
    if (!(form instanceof HTMLFormElement)) return;
    form.setAttribute("aria-disabled", String(!enabled));
    for (const control of form.elements) {
      if (control instanceof HTMLInputElement || control instanceof HTMLSelectElement || control instanceof HTMLTextAreaElement || control instanceof HTMLButtonElement) {
        control.disabled = !enabled;
      }
    }
  }

  function setAuthState(state, message) {
    if (authState instanceof HTMLElement) {
      authState.dataset.state = state;
      authState.textContent = message;
    }
  }

  function setResult(tone, message) {
    if (result instanceof HTMLElement) {
      result.dataset.tone = tone;
      result.textContent = message;
    }
  }

  async function loadSession() {
    try {
      const response = await fetch("/api/v1/auth/session", {
        credentials: "same-origin",
        headers: { Accept: "application/json" },
      });
      if (response.status === 401 || response.status === 404) {
        setAuthState("signed-out", labels.signedOut);
        return;
      }
      if (!response.ok) throw new Error("session unavailable");
      const session = await response.json();
      if (typeof session.csrf_token !== "string" || session.csrf_token.length < 16) {
        throw new Error("invalid session response");
      }
      csrfToken = session.csrf_token;
      setFormEnabled(true);
      if (login instanceof HTMLElement) login.hidden = true;
      setAuthState("ready", labels.ready);
    } catch (_error) {
      setAuthState("error", labels.checkingFailed);
    }
  }

  if (form instanceof HTMLFormElement) {
    setFormEnabled(false);
    form.addEventListener("submit", async (event) => {
      event.preventDefault();
      if (!csrfToken || !form.reportValidity()) return;
      const data = new FormData(form);
      const appVersion = String(data.get("app_version") ?? "").trim();
      const payload = {
        category: String(data.get("category") ?? ""),
        platform: String(data.get("platform") ?? ""),
        app_version: appVersion || null,
        title: String(data.get("title") ?? ""),
        description: String(data.get("description") ?? ""),
        redaction_confirmed: data.get("redaction_confirmed") === "on",
      };
      setFormEnabled(false);
      setResult("progress", labels.submitting);
      try {
        const response = await fetch("/api/v1/feedback", {
          method: "POST",
          credentials: "same-origin",
          headers: {
            Accept: "application/json",
            "Content-Type": "application/json",
            "X-CSRF-Token": csrfToken,
          },
          body: JSON.stringify(payload),
        });
        if (response.status === 401 || response.status === 403) {
          csrfToken = "";
          if (login instanceof HTMLElement) login.hidden = false;
          setAuthState("signed-out", labels.unauthorized);
          setResult("error", labels.unauthorized);
          return;
        }
        if (response.status === 400 || response.status === 422) {
          setResult("error", labels.invalid);
          return;
        }
        if (response.status === 429) {
          setResult("error", labels.limited);
          return;
        }
        if (!response.ok) throw new Error("feedback submission failed");
        const created = await response.json();
        const reference = typeof created.id === "string" ? created.id : "—";
        form.reset();
        setResult("success", `${labels.success} ${reference}`);
      } catch (_error) {
        setResult("error", labels.failed);
      } finally {
        if (csrfToken) setFormEnabled(true);
        if (submit instanceof HTMLButtonElement && !csrfToken) submit.disabled = true;
      }
    });
  }

  void loadSession();
}
