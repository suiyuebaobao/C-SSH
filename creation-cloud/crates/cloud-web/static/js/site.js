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

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Element)) {
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
