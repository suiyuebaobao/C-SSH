// 首页二维码交互：用一次性事件代理管理居中放大、焦点约束与 HTMX 换页清理。
const homeQrWidgetSelector = "[data-home-qr-widget]";
const homeQrEndpoint = "/api/v1/site-media/home-qr";
const homeQrExpandedClass = "home-qr-expanded";
let expandedHomeQrWidget = null;
let homeQrFocusReturn = null;
let homeQrInertedElements = [];

function homeQrUnavailableLabel() {
  return document.documentElement.lang.toLowerCase().startsWith("zh")
    ? "二维码暂时不可用"
    : "QR code temporarily unavailable";
}

function validHomeQrPayload(payload) {
  if (!payload || typeof payload !== "object") {
    return null;
  }

  const id = typeof payload.id === "string" ? payload.id.toLowerCase() : "";
  const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
  if (!uuid.test(id) || payload.content_type !== "image/png") {
    return null;
  }

  let contentUrl;
  try {
    contentUrl = new URL(payload.content_url, window.location.origin);
  } catch {
    return null;
  }
  if (
    contentUrl.origin !== window.location.origin ||
    contentUrl.search !== "" ||
    contentUrl.hash !== "" ||
    contentUrl.pathname !== `/api/v1/site-media/${id}/content`
  ) {
    return null;
  }

  const width = Number(payload.width);
  const height = Number(payload.height);
  if (!Number.isInteger(width) || width < 128 || width > 2048 || width !== height) {
    return null;
  }

  const language = document.documentElement.lang.toLowerCase();
  const alt = language.startsWith("zh") ? payload.alt_zh : payload.alt_en;
  if (typeof alt !== "string" || alt.trim() === "" || alt.length > 200) {
    return null;
  }

  return { alt: alt.trim(), contentUrl: contentUrl.pathname, height, width };
}

function setHomeQrUnavailable(widget) {
  widget.dataset.homeQrState = "unavailable";
  const label = homeQrUnavailableLabel();
  const status = widget.querySelector("[data-home-qr-status-text]");
  const placeholder = widget.querySelector(".home-qr-placeholder");
  const waiting = widget.querySelector(".home-qr-waiting small");
  if (status) {
    status.textContent = label;
  }
  if (placeholder) {
    placeholder.setAttribute("aria-label", label);
  }
  if (waiting) {
    waiting.textContent = "UNAVAILABLE";
  }
}

async function hydrateHomeQr(widget) {
  if (!(widget instanceof HTMLElement) || widget.dataset.homeQrLoad) {
    return;
  }
  widget.dataset.homeQrLoad = "loading";
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), 8000);
  try {
    const response = await fetch(homeQrEndpoint, {
      credentials: "same-origin",
      headers: { accept: "application/json" },
      signal: controller.signal,
    });
    if (response.status === 404) {
      widget.dataset.homeQrLoad = "complete";
      return;
    }
    if (!response.ok) {
      throw new Error("home QR metadata unavailable");
    }
    const media = validHomeQrPayload(await response.json());
    if (!media) {
      throw new Error("home QR metadata invalid");
    }

    const placeholder = widget.querySelector(".home-qr-placeholder");
    const status = widget.querySelector("[data-home-qr-status-text]");
    if (!placeholder || !status) {
      throw new Error("home QR widget incomplete");
    }
    const image = document.createElement("img");
    image.className = "home-qr-image";
    image.src = media.contentUrl;
    image.alt = media.alt;
    image.width = media.width;
    image.height = media.height;
    image.decoding = "async";
    image.referrerPolicy = "same-origin";
    placeholder.replaceWith(image);
    status.textContent = status.closest(".home-qr-status")?.dataset.readyLabel ?? "";
    widget.dataset.homeQrState = "ready";
    widget.dataset.homeQrLoad = "complete";
  } catch {
    widget.dataset.homeQrLoad = "failed";
    setHomeQrUnavailable(widget);
  } finally {
    window.clearTimeout(timeout);
  }
}

function getHomeQrParts(widget) {
  if (!(widget instanceof HTMLElement)) {
    return null;
  }

  const trigger = widget.querySelector("[data-home-qr-trigger]");
  const closeButton = widget.querySelector("[data-home-qr-close]");
  if (!(trigger instanceof HTMLButtonElement) || !(closeButton instanceof HTMLButtonElement)) {
    return null;
  }

  return { trigger, closeButton };
}

function homeQrIsExpanded(widget) {
  return widget?.dataset.homeQrExpanded === "true";
}

function setHomeQrBackgroundInert(widget) {
  homeQrInertedElements = [];
  for (const child of document.body.children) {
    if (child !== widget && child instanceof HTMLElement && !child.inert) {
      child.inert = true;
      homeQrInertedElements.push(child);
    }
  }
}

function restoreHomeQrBackground() {
  for (const element of homeQrInertedElements) {
    if (element.isConnected) {
      element.inert = false;
    }
  }
  homeQrInertedElements = [];
}

function setHomeQrExpanded(widget, expanded, { restoreFocus = true } = {}) {
  const parts = getHomeQrParts(widget);
  if (!parts) {
    if (!expanded) {
      restoreHomeQrBackground();
      document.body.classList.remove(homeQrExpandedClass);
      expandedHomeQrWidget = null;
      homeQrFocusReturn = null;
    }
    return false;
  }

  if (expanded) {
    if (expandedHomeQrWidget && expandedHomeQrWidget !== widget) {
      setHomeQrExpanded(expandedHomeQrWidget, false, { restoreFocus: false });
    }

    expandedHomeQrWidget = widget;
    homeQrFocusReturn = parts.trigger;
    widget.dataset.homeQrExpanded = "true";
    widget.setAttribute("role", "dialog");
    widget.setAttribute("aria-modal", "true");
    widget.setAttribute("aria-labelledby", "home-qr-title");
    widget.setAttribute("aria-describedby", "home-qr-status home-qr-note");
    parts.trigger.setAttribute("aria-expanded", "true");
    parts.trigger.setAttribute("aria-label", parts.trigger.dataset.closeLabel ?? "");
    parts.closeButton.hidden = false;
    setHomeQrBackgroundInert(widget);
    document.body.classList.add(homeQrExpandedClass);
    window.requestAnimationFrame(() => {
      if (homeQrIsExpanded(widget)) {
        parts.closeButton.focus();
      }
    });
    return true;
  }

  widget.dataset.homeQrExpanded = "false";
  widget.removeAttribute("role");
  widget.removeAttribute("aria-modal");
  widget.removeAttribute("aria-labelledby");
  widget.removeAttribute("aria-describedby");
  parts.trigger.setAttribute("aria-expanded", "false");
  parts.trigger.setAttribute("aria-label", parts.trigger.dataset.openLabel ?? "");
  parts.closeButton.hidden = true;
  restoreHomeQrBackground();
  document.body.classList.remove(homeQrExpandedClass);

  const focusTarget = homeQrFocusReturn ?? parts.trigger;
  expandedHomeQrWidget = null;
  homeQrFocusReturn = null;
  if (restoreFocus && focusTarget.isConnected) {
    focusTarget.focus({ preventScroll: true });
  }
  return true;
}

function closeExpandedHomeQr({ restoreFocus = true } = {}) {
  if (!expandedHomeQrWidget) {
    restoreHomeQrBackground();
    document.body.classList.remove(homeQrExpandedClass);
    return false;
  }

  return setHomeQrExpanded(expandedHomeQrWidget, false, { restoreFocus });
}

function synchronizeHomeQr() {
  if (expandedHomeQrWidget && !expandedHomeQrWidget.isConnected) {
    expandedHomeQrWidget = null;
    homeQrFocusReturn = null;
    restoreHomeQrBackground();
  }

  for (const widget of document.querySelectorAll(homeQrWidgetSelector)) {
    void hydrateHomeQr(widget);
    if (homeQrIsExpanded(widget) && widget !== expandedHomeQrWidget) {
      setHomeQrExpanded(widget, false, { restoreFocus: false });
    }
  }

  if (!document.querySelector(`${homeQrWidgetSelector}[data-home-qr-expanded="true"]`)) {
    document.body.classList.remove(homeQrExpandedClass);
  }
}

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) {
    return;
  }

  const closeButton = event.target.closest("[data-home-qr-close]");
  if (closeButton) {
    closeExpandedHomeQr();
    return;
  }

  const trigger = event.target.closest("[data-home-qr-trigger]");
  if (trigger) {
    const widget = trigger.closest(homeQrWidgetSelector);
    if (widget) {
      setHomeQrExpanded(widget, !homeQrIsExpanded(widget));
    }
    return;
  }

  const card = event.target.closest(".home-qr-card");
  if (expandedHomeQrWidget && card && expandedHomeQrWidget.contains(card)) {
    closeExpandedHomeQr();
    return;
  }

  if (expandedHomeQrWidget && event.target === expandedHomeQrWidget) {
    closeExpandedHomeQr();
  }
});

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    closeExpandedHomeQr();
    return;
  }

  if (event.key !== "Tab" || !expandedHomeQrWidget) {
    return;
  }

  const parts = getHomeQrParts(expandedHomeQrWidget);
  if (!parts) {
    return;
  }

  const first = parts.trigger;
  const last = parts.closeButton;
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault();
    first.focus();
  } else if (document.activeElement !== first && document.activeElement !== last) {
    event.preventDefault();
    last.focus();
  }
});

document.addEventListener("DOMContentLoaded", synchronizeHomeQr);
document.addEventListener("htmx:beforeSwap", () => closeExpandedHomeQr({ restoreFocus: false }));
document.addEventListener("htmx:beforeHistorySave", () => closeExpandedHomeQr({ restoreFocus: false }));
document.addEventListener("htmx:load", synchronizeHomeQr);
window.addEventListener("pagehide", () => closeExpandedHomeQr({ restoreFocus: false }));
