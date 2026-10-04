export default defineEventHandler((event) => {
  // 发布/撤下必须即时可见；SSR不共享用户Cookie或错误页面缓存。
  if (!getRequestURL(event).pathname.startsWith("/_nuxt/")) {
    setResponseHeader(event, "Cache-Control", "no-store");
  }
  if (
    /^\/(?:en\/)?(?:login|register|verify-email|verify-login|forgot-password|reset-password|console|admin)(?:\/|$)/.test(
      getRequestURL(event).pathname,
    )
  ) {
    setResponseHeader(event, "X-Robots-Tag", "noindex, nofollow");
  }
  setResponseHeader(event, "X-Content-Type-Options", "nosniff");
  setResponseHeader(
    event,
    "Referrer-Policy",
    "strict-origin-when-cross-origin",
  );
});
