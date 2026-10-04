import type { SiteDocument } from "~/types/site";

export function useSiteDocument(key: string, locale: "zh-CN" | "en") {
  const config = useRuntimeConfig();
  return useAsyncData(`site:${key}:${locale}`, async () => {
    // 普通$fetch不继承SSR请求Cookie；浏览器请求保持同源。禁止接受客户端指定上游。
    const baseURL = import.meta.server ? config.rustOrigin : undefined;
    try {
      return await $fetch<SiteDocument>(`/api/v1/site/pages/${key}`, {
        baseURL,
        query: { locale },
        retry: 0,
        timeout: 10000,
        credentials: "omit",
      });
    } catch (cause) {
      const status = (cause as { status?: number }).status;
      throw createError({
        statusCode: status === 404 ? 404 : 503,
        statusMessage: "Page unavailable",
      });
    }
  });
}
