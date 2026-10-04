import type { SiteDocument } from "../../app/types/site";
import type { H3Event } from "h3";
export async function publicSite(event: H3Event, page: string) {
  const config = useRuntimeConfig(event);
  return $fetch<SiteDocument>(`/api/v1/site/pages/${page}`, {
    baseURL: config.rustOrigin,
    query: { locale: "zh-CN" },
    credentials: "omit",
    retry: 0,
    timeout: 10000,
  });
}
export function xml(value: string) {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}
