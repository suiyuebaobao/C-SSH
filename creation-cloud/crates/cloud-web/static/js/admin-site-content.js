// 站点内容编辑器只提示本地未保存状态；保存、预览和发布仍由服务端状态机决定。
function markSiteContentDirty(form) {
  if (!(form instanceof HTMLFormElement) || form.dataset.siteContentDirty === "true") {
    return;
  }
  form.dataset.siteContentDirty = "true";
  const editor = form.closest("[data-site-content-editor]");
  const marker = editor?.querySelector("[data-site-content-dirty]");
  if (marker instanceof HTMLElement) {
    marker.hidden = false;
  }
  const preview = editor?.querySelector("[data-site-content-preview]");
  if (preview instanceof HTMLAnchorElement) {
    preview.dataset.unsaved = "true";
    preview.title = document.documentElement.lang.toLowerCase().startsWith("en")
      ? "Preview shows the last saved revision."
      : "预览展示最后一次已保存的版本。";
  }
}

document.addEventListener("input", (event) => {
  const form = event.target instanceof Element
    ? event.target.closest("[data-site-content-form]")
    : null;
  markSiteContentDirty(form);
});

document.addEventListener("change", (event) => {
  const form = event.target instanceof Element
    ? event.target.closest("[data-site-content-form]")
    : null;
  markSiteContentDirty(form);
});
