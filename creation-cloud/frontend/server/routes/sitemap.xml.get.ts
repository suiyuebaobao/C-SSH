import { publicSite, xml } from "../utils/public-site";
export default defineEventHandler(async (event) => {
  const [base, catalog] = await Promise.all([
    publicSite(event, "security"),
    publicSite(event, "downloads"),
  ]);
  if (!base.seo.canonical_url) throw createError({ statusCode: 503 });
  const origin = new URL(base.seo.canonical_url).origin;
  const paths = [
    "/",
    "/docs/getting-started",
    "/tutorials",
    "/security",
    "/faq",
    "/feedback",
  ];
  if (catalog.catalog?.releases.length) paths.push("/downloads", "/changelog");
  const rows = paths.flatMap((path) => {
    const zh = new URL(path, origin).href;
    const en = new URL(`/en${path === "/" ? "" : path}`, origin).href;
    return [zh, en].map(
      (current) =>
        `<url><loc>${xml(current)}</loc><xhtml:link rel="alternate" hreflang="zh-CN" href="${xml(zh)}"/><xhtml:link rel="alternate" hreflang="en" href="${xml(en)}"/><xhtml:link rel="alternate" hreflang="x-default" href="${xml(zh)}"/></url>`,
    );
  });
  setResponseHeader(event, "Content-Type", "application/xml; charset=utf-8");
  return `<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">${rows.join("")}</urlset>`;
});
