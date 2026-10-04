import type { SiteDocument } from "~/types/site";

export function usePageSeo(document: Ref<SiteDocument | undefined>) {
  useHead(() => {
    if (!document.value) return {};
    const { view, seo } = document.value;
    const meta = [
      { name: "description", content: view.page.meta_description },
      { name: "robots", content: seo.robots },
      ...Object.entries({
        "og:title": seo.open_graph_title,
        "og:description": seo.open_graph_description,
        "og:url": seo.open_graph_url,
        "og:locale": seo.open_graph_locale,
        "og:locale:alternate": seo.open_graph_alternate_locale,
      })
        .filter(([, value]) => value)
        .map(([property, content]) => ({ property, content: content! })),
      ...Object.entries({
        "google-site-verification": seo.google_site_verification,
        "baidu-site-verification": seo.baidu_site_verification,
        keywords: seo.keywords,
      })
        .filter(([, value]) => value)
        .map(([name, content]) => ({ name, content: content! })),
    ];
    const link = [
      ...(seo.canonical_url
        ? [{ rel: "canonical" as const, href: seo.canonical_url }]
        : []),
      ...Object.entries({
        "zh-CN": seo.alternate_zh_url,
        en: seo.alternate_en_url,
        "x-default": seo.alternate_default_url,
      })
        .filter(([, href]) => href)
        .map(([hreflang, href]) => ({
          rel: "alternate" as const,
          hreflang,
          href: href!,
        })),
    ];
    return {
      title: view.page.meta_title,
      htmlAttrs: { lang: view.shell.html_lang },
      meta,
      link,
      // JSON-LD只作JSON数据；转义HTML边界，避免动态正文结束script元素。
      script: seo.json_ld
        ? [
            {
              type: "application/ld+json",
              innerHTML: seo.json_ld
                .replace(/</g, "\\u003c")
                .replace(/>/g, "\\u003e")
                .replace(/&/g, "\\u0026"),
            },
          ]
        : [],
    };
  });
}
