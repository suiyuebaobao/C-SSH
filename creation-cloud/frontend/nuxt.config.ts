export default defineNuxtConfig({
  compatibilityDate: "2026-10-02",
  devtools: { enabled: false },
  modules: ["@pinia/nuxt"],
  css: [
    "~/assets/css/foundation.css",
    "~/assets/css/public-pages.css",
    "~/assets/css/workspace.css",
  ],
  runtimeConfig: {
    // 仅服务器可见；SSR只读取公开闭集接口，不转发访问者认证信息。
    rustOrigin: "http://127.0.0.1:18088",
  },
  routeRules: {
    ...Object.fromEntries(
      [
        "login",
        "register",
        "verify-email",
        "verify-login",
        "forgot-password",
        "reset-password",
      ].flatMap((path) => [
        [`/${path}`, { ssr: false }],
        [`/en/${path}`, { ssr: false }],
      ]),
    ),
    "/console": { ssr: false },
    "/console/**": { ssr: false },
    "/admin": { ssr: false },
    "/admin/**": { ssr: false },
  },
  typescript: { strict: true },
  nitro: { preset: "node-server" },
  sourcemap: { server: false, client: false },
  app: {
    head: {
      link: [
        { rel: "icon", type: "image/png", href: "/static/img/brand-c.png" },
      ],
      meta: [{ name: "theme-color", content: "#090b0d" }],
    },
  },
});
