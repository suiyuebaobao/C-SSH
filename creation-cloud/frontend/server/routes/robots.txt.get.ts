import { publicSite } from "../utils/public-site";
export default defineEventHandler(async (event) => {
  const data = await publicSite(event, "security");
  if (!data.seo.canonical_url) throw createError({ statusCode: 503 });
  const sitemap = new URL("/sitemap.xml", data.seo.canonical_url).href;
  setResponseHeader(event, "Content-Type", "text/plain; charset=utf-8");
  return `User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /api/\nDisallow: /console\nDisallow: /health/\nDisallow: /_health\nDisallow: /login\nDisallow: /register\nDisallow: /en/login\nDisallow: /en/register\nSitemap: ${sitemap}\n`;
});
