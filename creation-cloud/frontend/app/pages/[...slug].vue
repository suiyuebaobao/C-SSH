<script setup lang="ts">
const route = useRoute();
const routes: Record<string, string> = {
  "/": "home",
  "/downloads": "downloads",
  "/changelog": "changelog",
  "/tutorials": "tutorials",
  "/docs/getting-started": "documentation",
  "/security": "security",
  "/faq": "faq",
};
const en = route.path === "/en" || route.path.startsWith("/en/");
const path = en ? route.path.slice(3) || "/" : route.path;
const key = routes[path];
if (!key)
  throw createError({ statusCode: 404, statusMessage: "Page not found" });
const { data, error } = await useSiteDocument(key, en ? "en" : "zh-CN");
if (error.value || !data.value)
  throw createError({
    statusCode: error.value?.statusCode ?? 503,
    statusMessage: "Page unavailable",
  });
usePageSeo(data);
</script>
<template>
  <SiteFrame v-if="data" :shell="data.view.shell">
    <HomeContent v-if="key === 'home'" :document="data" />
    <DownloadContent v-else-if="key === 'downloads'" :document="data" />
    <ChangelogContent v-else-if="key === 'changelog'" :document="data" />
    <TutorialContent v-else-if="key === 'tutorials'" :document="data" />
    <DocumentationContent
      v-else-if="key === 'documentation'"
      :document="data"
    />
    <main v-else id="main-content" class="container narrow">
      <PageHeading :page="data.view.page" /><ContentSections
        :sections="data.view.page.sections"
      /><FaqList :items="data.view.page.faqs" />
    </main>
  </SiteFrame>
</template>
