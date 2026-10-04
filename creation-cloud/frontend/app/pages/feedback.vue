<script setup lang="ts">
definePageMeta({ alias: "/en/feedback" });
const { en, t, publicPath } = useLocale();
const session = useSessionStore();
const checking = ref(true);
const sessionError = ref("");
const { data, error } = await useSiteDocument(
  "feedback",
  en.value ? "en" : "zh-CN",
);
if (error.value || !data.value)
  throw createError({ statusCode: 503, statusMessage: "Page unavailable" });
usePageSeo(data);
onMounted(async () => {
  try {
    if (!session.current) await session.refresh();
  } catch (cause) {
    if ((cause as { status?: number }).status !== 401)
      sessionError.value = cloudError(cause, en.value);
  } finally {
    checking.value = false;
  }
});
</script>
<template>
  <SiteFrame v-if="data" :shell="data.view.shell"
    ><main id="main-content" class="container narrow">
      <PageHeading :page="data.view.page" /><ClientOnly
        ><WorkspaceFeedbackForm v-if="session.current" />
        <p v-else-if="checking">
          {{ t("检查登录状态…", "Checking session…") }}
        </p>
        <p v-else-if="sessionError" role="alert">{{ sessionError }}</p>
        <div v-else class="notice">
          <p>
            {{
              t(
                "登录后可向我们提交反馈并查看处理状态。",
                "Sign in to submit feedback and track its status.",
              )
            }}
          </p>
          <NuxtLink
            class="button primary"
            :to="`${publicPath('/login')}?next=/feedback`"
            >{{ t("登录并提交反馈", "Sign in to send feedback") }}</NuxtLink
          >
        </div></ClientOnly
      ><ContentSections :sections="data.view.page.sections" /></main
  ></SiteFrame>
</template>
