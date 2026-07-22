// 管理后台交互：注入内存中的 CSRF、确认危险动作、注销并恢复局部刷新焦点。
let pendingAdminConfirmation = null;

function adminShell() {
  return document.querySelector("[data-admin-shell]");
}

function adminCsrfToken() {
  const shell = adminShell();
  return shell instanceof HTMLElement ? shell.dataset.csrfToken ?? "" : "";
}

function adminIsEnglish() {
  return document.documentElement.lang.toLowerCase().startsWith("en");
}

function adminGlobalFeedback() {
  return document.querySelector("[data-admin-global-feedback]");
}

function adminStatusMessage(status) {
  const english = adminIsEnglish();
  const messages = english
    ? {
        400: "The request format is invalid.",
        401: "Your session has expired. Sign in again to continue.",
        403: "The action was rejected by the permission or CSRF check.",
        404: "The requested record no longer exists.",
        409: "The action conflicts with the current record state.",
        413: "The uploaded file is too large.",
        422: "Review the submitted values and try again.",
        500: "The service is temporarily unavailable.",
      }
    : {
        400: "请求格式无效。",
        401: "会话已失效，请重新登录后继续。",
        403: "权限或 CSRF 校验拒绝了此操作。",
        404: "请求的记录已不存在。",
        409: "该操作与记录当前状态冲突。",
        413: "上传文件过大。",
        422: "请检查提交内容后重试。",
        500: "服务暂时不可用。",
      };
  return messages[status] ?? (english ? `Request failed (${status}).` : `请求失败（${status}）。`);
}

function adminSameOriginResponse(xhr) {
  try {
    return new URL(xhr.responseURL || window.location.href, window.location.href).origin === window.location.origin;
  } catch {
    return false;
  }
}

function adminResponseMessage(xhr) {
  const fallback = adminStatusMessage(xhr.status);
  if (!adminSameOriginResponse(xhr)) {
    return fallback;
  }

  const contentType = xhr.getResponseHeader("content-type") ?? "";
  const responseText = typeof xhr.responseText === "string" ? xhr.responseText : "";
  if (contentType.includes("application/json") && responseText.length <= 16_384) {
    try {
      const body = JSON.parse(responseText);
      const message = typeof body?.message === "string" ? body.message.trim() : "";
      const code = typeof body?.code === "string" ? body.code.trim() : "";
      if (!adminIsEnglish() && message) {
        return message.slice(0, 512);
      }
      if (code) {
        return `${fallback} [${code.slice(0, 64)}]`;
      }
    } catch {
      return fallback;
    }
  }

  if (contentType.includes("text/html") && responseText.length <= 65_536) {
    const parsed = new DOMParser().parseFromString(responseText, "text/html");
    const message = parsed.body.textContent?.replace(/\s+/g, " ").trim();
    if (message) {
      return message.slice(0, 512);
    }
  }
  return fallback;
}

function adminShowResponseError(xhr) {
  const feedback = adminGlobalFeedback();
  if (!(feedback instanceof HTMLElement)) {
    return;
  }
  feedback.replaceChildren();
  feedback.textContent = adminResponseMessage(xhr);
  feedback.setAttribute("data-tone", "error");
  feedback.setAttribute("tabindex", "-1");
  if (xhr.status === 401) {
    const link = document.createElement("a");
    const next = `${window.location.pathname}${window.location.search}`;
    link.href = `/login?next=${encodeURIComponent(next)}`;
    link.textContent = adminIsEnglish() ? " Sign in" : " 重新登录";
    feedback.append(link);
  }
  feedback.focus({ preventScroll: true });
}

document.addEventListener("htmx:configRequest", (event) => {
  const token = adminCsrfToken();
  if (token && event.detail?.headers) {
    event.detail.headers["x-csrf-token"] = token;
  }
});

document.addEventListener("htmx:responseError", (event) => {
  const xhr = event.detail?.xhr;
  if (xhr instanceof XMLHttpRequest) {
    adminShowResponseError(xhr);
  }
});

document.addEventListener("htmx:confirm", (event) => {
  const message = event.detail?.question;
  const dialog = document.querySelector("[data-admin-confirm-dialog]");
  if (!message || !(dialog instanceof HTMLDialogElement)) {
    return;
  }

  event.preventDefault();
  pendingAdminConfirmation = event.detail;
  const messageNode = dialog.querySelector("[data-admin-confirm-message]");
  if (messageNode) {
    messageNode.textContent = message;
  }
  dialog.showModal();
  dialog.querySelector("[data-admin-confirm-accept]")?.focus();
});

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) {
    return;
  }

  if (event.target.closest("[data-admin-confirm-accept]")) {
    const detail = pendingAdminConfirmation;
    pendingAdminConfirmation = null;
    document.querySelector("[data-admin-confirm-dialog]")?.close();
    detail?.issueRequest(true);
    return;
  }

  if (event.target.closest("[data-admin-confirm-cancel]")) {
    pendingAdminConfirmation = null;
    document.querySelector("[data-admin-confirm-dialog]")?.close();
  }
});

document.addEventListener("click", async (event) => {
  const button = event.target instanceof Element ? event.target.closest("[data-admin-logout]") : null;
  if (!(button instanceof HTMLButtonElement)) {
    return;
  }

  button.disabled = true;
  try {
    const response = await fetch("/api/v1/auth/logout", {
      method: "POST",
      credentials: "same-origin",
      headers: { "x-csrf-token": adminCsrfToken() },
    });
    if (!response.ok && response.status !== 401) {
      throw new Error("logout_failed");
    }
    window.location.assign("/login");
  } catch {
    button.disabled = false;
    const feedback = adminGlobalFeedback();
    if (feedback) {
      feedback.textContent = button.dataset.errorLabel ?? "Unable to sign out.";
      feedback.setAttribute("data-tone", "error");
    }
  }
});

document.addEventListener("htmx:afterSwap", (event) => {
  const target = event.detail?.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }
  const focusTarget = target.querySelector("[data-admin-swap-focus]");
  if (focusTarget instanceof HTMLElement) {
    focusTarget.focus({ preventScroll: true });
  }
});

function adminMaintenanceTime(value) {
  if (typeof value !== "string" || value.length > 64) {
    return adminIsEnglish() ? "Never" : "暂无";
  }
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return adminIsEnglish() ? "Unknown" : "未知";
  }
  return new Intl.DateTimeFormat(adminIsEnglish() ? "en" : "zh-CN", {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(parsed);
}

function adminMaintenanceTone(status) {
  const outcome = status?.latest_attempt?.outcome;
  if (status?.active_run_id) {
    return "running";
  }
  if (Number(status?.consecutive_failures) > 0 || ["failed", "timed_out", "interrupted"].includes(outcome)) {
    return "error";
  }
  if (["missing", "stale", "invalid", "issues_detected"].includes(status?.last_observation)) {
    return "warning";
  }
  return "healthy";
}

function adminMaintenanceItem(status) {
  const english = adminIsEnglish();
  const item = document.createElement("article");
  item.className = "admin-maintenance-item";
  item.dataset.tone = adminMaintenanceTone(status);

  const heading = document.createElement("div");
  heading.className = "admin-maintenance-heading";
  const name = document.createElement("strong");
  name.textContent = typeof status?.task === "string" ? status.task.slice(0, 64) : "unknown-task";
  const outcome = document.createElement("span");
  outcome.className = "status-badge";
  outcome.textContent = status?.active_run_id
    ? (english ? "running" : "运行中")
    : (status?.latest_attempt?.outcome ?? (english ? "not run" : "尚未运行"));
  heading.append(name, outcome);

  const attempt = document.createElement("p");
  attempt.textContent = `${english ? "Latest attempt" : "最近尝试"}: ${adminMaintenanceTime(status?.latest_attempt?.started_at)}`;
  const success = document.createElement("p");
  success.textContent = `${english ? "Last success" : "最近成功"}: ${adminMaintenanceTime(status?.last_success_at)}`;
  const detail = document.createElement("small");
  const observation = status?.last_observation ?? (english ? "none" : "暂无");
  const failures = Number.isSafeInteger(status?.consecutive_failures) ? status.consecutive_failures : 0;
  detail.textContent = `${english ? "Observation" : "观察"}: ${observation} · ${english ? "Consecutive failures" : "连续失败"}: ${failures}`;
  item.append(heading, attempt, success, detail);
  return item;
}

async function adminLoadMaintenance(panel) {
  if (!(panel instanceof HTMLElement) || panel.dataset.loading === "true") {
    return;
  }
  const endpoint = panel.dataset.maintenanceEndpoint;
  const list = panel.querySelector("[data-maintenance-list]");
  const feedback = panel.querySelector("[data-maintenance-feedback]");
  const refresh = panel.querySelector("[data-maintenance-refresh]");
  if (!endpoint || !(list instanceof HTMLElement) || !(feedback instanceof HTMLElement)) {
    return;
  }
  panel.dataset.loading = "true";
  if (refresh instanceof HTMLButtonElement) {
    refresh.disabled = true;
  }
  try {
    const response = await fetch(endpoint, { credentials: "same-origin", headers: { Accept: "application/json" } });
    if (!response.ok) {
      throw new Error("maintenance_status_failed");
    }
    const statuses = await response.json();
    if (!Array.isArray(statuses) || statuses.length !== 5) {
      throw new Error("maintenance_status_shape");
    }
    list.replaceChildren(...statuses.map(adminMaintenanceItem));
    feedback.textContent = adminIsEnglish() ? "Maintenance status is current." : "维护状态已更新。";
    feedback.setAttribute("data-tone", "success");
  } catch {
    list.replaceChildren();
    feedback.textContent = adminIsEnglish() ? "Maintenance status is temporarily unavailable." : "维护状态暂时无法读取。";
    feedback.setAttribute("data-tone", "error");
  } finally {
    delete panel.dataset.loading;
    if (refresh instanceof HTMLButtonElement) {
      refresh.disabled = false;
    }
  }
}

document.addEventListener("DOMContentLoaded", () => {
  const panel = document.querySelector("[data-maintenance-panel]");
  adminLoadMaintenance(panel);
  panel?.querySelector("[data-maintenance-refresh]")?.addEventListener("click", () => {
    adminLoadMaintenance(panel);
  });
});
