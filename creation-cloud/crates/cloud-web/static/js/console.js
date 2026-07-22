// 用户控制台脚本：为同源 HTMX 写请求补充由服务端会话派生的 CSRF 请求头。
function consoleRoot() {
  return document.querySelector("[data-console-root]");
}

function csrfToken() {
  return consoleRoot()?.dataset.csrfToken ?? "";
}

document.addEventListener("htmx:configRequest", (event) => {
  const token = csrfToken();
  if (token && event.detail?.headers) {
    event.detail.headers["X-CSRF-Token"] = token;
  }
});
