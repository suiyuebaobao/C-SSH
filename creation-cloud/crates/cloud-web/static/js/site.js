// 全站交互入口：以一次性事件代理维护响应式菜单，兼容 HTMX 替换页面主体。
document.documentElement.classList.add("js");

const desktopMedia = window.matchMedia("(min-width: 1201px)");
let activeMenuButton = null;

function getMenuElements() {
  const button = document.querySelector("[data-menu-button]");
  const navigation = document.querySelector("[data-site-navigation]");

  if (!button || !navigation) {
    return null;
  }

  return { button, navigation };
}

function menuIsOpen(button) {
  return button.getAttribute("aria-expanded") === "true";
}

function setMenuState(elements, open) {
  elements.button.setAttribute("aria-expanded", String(open));
  elements.navigation.dataset.open = String(open);
}

function closeMenu({ restoreFocus = false } = {}) {
  const elements = getMenuElements();

  if (!elements || !menuIsOpen(elements.button)) {
    return false;
  }

  setMenuState(elements, false);
  if (restoreFocus && elements.button.isConnected) {
    elements.button.focus();
  }
  return true;
}

function synchronizeMenu() {
  const elements = getMenuElements();

  if (!elements || elements.button === activeMenuButton) {
    return;
  }

  activeMenuButton = elements.button;
  setMenuState(elements, false);
}

function accountFormFromHtmxEvent(event) {
  const candidates = [
    event.detail?.requestConfig?.elt,
    event.detail?.elt,
    event.detail?.target,
    event.target,
  ];

  for (const candidate of candidates) {
    if (candidate instanceof HTMLFormElement && candidate.matches(".account-form")) {
      return candidate;
    }
    if (candidate instanceof Element) {
      const form = candidate.closest("form.account-form");
      if (form instanceof HTMLFormElement) {
        return form;
      }
    }
  }
  return null;
}

function accountErrorMessage(status, networkFailure = false, captchaExpected = false) {
  const english = document.documentElement.lang.toLowerCase().startsWith("en");
  if (networkFailure) {
    return english
      ? "Unable to reach the service. Check your connection and try again."
      : "暂时无法连接服务，请检查网络后重试。";
  }
  if (status === 401) {
    if (captchaExpected) {
      return english
        ? "The account, password, or visual CAPTCHA is incorrect."
        : "账号、密码或图形验证码错误。";
    }
    return english ? "The account or password is incorrect." : "账号或密码错误。";
  }
  if (status === 429) {
    return english
      ? "Too many attempts. Please wait and try again."
      : "尝试次数过多，请稍后再试。";
  }
  if (status === 400 || status === 422) {
    return english
      ? "Check the information you entered and try again."
      : "请检查填写内容后重试。";
  }
  if (status >= 500) {
    return english
      ? "The service is temporarily unavailable. Please try again later."
      : "服务暂时不可用，请稍后重试。";
  }
  return english
    ? "The request could not be completed. Please try again."
    : "请求未能完成，请稍后重试。";
}

function safeResponseMessage(event) {
  const responseText = event.detail?.xhr?.responseText;
  if (typeof responseText !== "string" || responseText.length > 2048) {
    return null;
  }
  try {
    const message = JSON.parse(responseText)?.message;
    if (
      typeof message === "string" &&
      message.length > 0 &&
      message.length <= 256 &&
      !/[\u0000-\u001f\u007f]/.test(message)
    ) {
      return message;
    }
  } catch (_error) {
    return null;
  }
  return null;
}

function passwordResetErrorMessage(event, status) {
  const english = document.documentElement.lang.toLowerCase().startsWith("en");
  if (status === 401) {
    return english
      ? "The code is invalid, expired, or has reached the attempt limit."
      : "验证码无效、已过期或已达到尝试上限。";
  }
  if (status === 400 || status === 422) {
    if (!english) {
      const message = safeResponseMessage(event);
      if (message) {
        return message;
      }
    }
    return english
      ? "Check that both passwords match, use 12 to 128 characters, and choose a new password."
      : "请确认两次密码一致、长度为 12 至 128 个字符，且未与当前密码重复。";
  }
  return null;
}

function accountShowError(event, networkFailure = false) {
  const form = accountFormFromHtmxEvent(event);
  const feedback = form?.querySelector(".form-feedback");
  if (!(feedback instanceof HTMLElement)) {
    return;
  }
  const status = Number(event.detail?.xhr?.status) || 0;
  const contextualMessage =
    !networkFailure && form?.dataset.errorContext === "password-reset"
      ? passwordResetErrorMessage(event, status)
      : null;
  const captchaButton = form?.querySelector("[data-captcha-refresh]");
  if (!networkFailure && captchaButton instanceof HTMLButtonElement) {
    captchaButton.dataset.refreshAfterRequest = "true";
  }
  feedback.textContent =
    contextualMessage ||
    accountErrorMessage(
      status,
      networkFailure,
      captchaButton instanceof HTMLButtonElement,
    );
  feedback.setAttribute("data-tone", "error");
  feedback.setAttribute("tabindex", "-1");
  feedback.focus({ preventScroll: true });
}

function refreshCaptcha(button) {
  if (button.disabled) {
    return;
  }

  const image = button.querySelector("[data-captcha-image]");
  const url = button.dataset.captchaUrl;
  if (!(image instanceof HTMLImageElement) || !url) {
    return;
  }

  button.disabled = true;
  button.setAttribute("aria-busy", "true");
  const finish = () => {
    image.removeEventListener("load", finish);
    image.removeEventListener("error", finish);
    button.disabled = false;
    button.removeAttribute("aria-busy");
  };
  image.addEventListener("load", finish);
  image.addEventListener("error", finish);

  const source = new URL(url, window.location.href);
  source.searchParams.set("refresh", Date.now().toString());
  image.src = `${source.pathname}${source.search}`;

  const input = button
    .closest("form.account-form")
    ?.querySelector('input[name="captcha_code"]');
  if (input instanceof HTMLInputElement) {
    input.value = "";
    input.focus({ preventScroll: true });
  }
}

function refreshCaptchaAfterRequest(event) {
  const captchaButton = accountFormFromHtmxEvent(event)?.querySelector(
    "[data-captcha-refresh]",
  );
  if (
    !(captchaButton instanceof HTMLButtonElement) ||
    captchaButton.dataset.refreshAfterRequest !== "true"
  ) {
    return;
  }
  delete captchaButton.dataset.refreshAfterRequest;
  refreshCaptcha(captchaButton);
}

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) {
    return;
  }

  const captchaButton = event.target.closest("[data-captcha-refresh]");
  if (captchaButton instanceof HTMLButtonElement) {
    refreshCaptcha(captchaButton);
    return;
  }

  const button = event.target.closest("[data-menu-button]");
  if (button) {
    const elements = getMenuElements();
    if (elements && elements.button === button) {
      setMenuState(elements, !menuIsOpen(button));
    }
    return;
  }

  if (event.target.closest("[data-site-navigation] a")) {
    closeMenu();
    return;
  }

  if (!event.target.closest("[data-site-navigation]")) {
    closeMenu();
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeMenu({ restoreFocus: true });
  }
});

desktopMedia.addEventListener("change", (event) => {
  if (event.matches) {
    closeMenu();
  }
});

document.addEventListener("DOMContentLoaded", synchronizeMenu);
document.addEventListener("htmx:beforeSwap", () => closeMenu());
document.addEventListener("htmx:load", synchronizeMenu);
document.addEventListener("htmx:beforeRequest", (event) => {
  const feedback = accountFormFromHtmxEvent(event)?.querySelector(".form-feedback");
  if (feedback instanceof HTMLElement) {
    feedback.textContent = "";
    feedback.removeAttribute("data-tone");
    feedback.removeAttribute("tabindex");
  }
});
document.addEventListener("htmx:responseError", (event) => accountShowError(event));
document.addEventListener("htmx:afterRequest", refreshCaptchaAfterRequest);
document.addEventListener("htmx:sendError", (event) => accountShowError(event, true));
document.addEventListener("htmx:timeout", (event) => accountShowError(event, true));
