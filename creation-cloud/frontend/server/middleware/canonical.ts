const paths = new Set([
  "/",
  "/downloads",
  "/changelog",
  "/tutorials",
  "/en/tutorials",
  "/docs/getting-started",
  "/en/docs/getting-started",
  "/security",
  "/faq",
]);

export default defineEventHandler((event) => {
  const url = getRequestURL(event);
  if (event.method !== "GET") return;
  const language = [...url.searchParams].find(
    ([name]) => name.toLowerCase() === "lang",
  )?.[1];
  if (url.pathname === "/en/") return sendRedirect(event, "/en", 301);
  if (/^\/(en\/)?(features|platforms)$/.test(url.pathname))
    return sendRedirect(
      event,
      `${language === undefined ? (url.pathname.startsWith("/en/") ? "/en" : "") : ["en", "en-us"].includes(language.trim().toLowerCase()) ? "/en" : ""}/docs/getting-started`,
      301,
    );
  if (!paths.has(url.pathname)) return;
  if (language === undefined) return;
  const en = ["en", "en-us"].includes(language.trim().toLowerCase());
  const path = url.pathname.replace(/^\/en(?=\/)/, "");
  return sendRedirect(event, en ? `/en${path === "/" ? "" : path}` : path, 301);
});
