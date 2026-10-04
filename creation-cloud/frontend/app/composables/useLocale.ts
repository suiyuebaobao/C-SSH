export function useLocale() {
  const route = useRoute();
  const en = computed(
    () =>
      route.path === "/en" ||
      route.path.startsWith("/en/") ||
      route.query.lang === "en",
  );
  const t = (zh: string, english: string) => (en.value ? english : zh);
  const publicPath = (path: string) =>
    en.value ? `/en${path === "/" ? "" : path}` : path;
  return { en, t, publicPath };
}
