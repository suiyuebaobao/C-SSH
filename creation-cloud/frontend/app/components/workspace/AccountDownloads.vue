<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
interface Release {
  id: string;
  version: string;
  assets: {
    id: string;
    file_name: string;
    platform: string;
    architecture: string;
    sources: { id: string; provider_name: string }[];
  }[];
}
const releases = ref<Release[]>([]);
const error = ref("");
onMounted(async () => {
  try {
    releases.value = await api("/api/v1/downloads/releases");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
const columns = computed(() => [
  { key: "version", label: t("版本", "Version") },
  { key: "file_name", label: t("文件", "File") },
  { key: "provider_name", label: t("来源", "Source") },
  { key: "occurred_at", label: t("时间", "Time") },
]);
</script>
<template>
  <h1>{{ t("下载与记录", "Downloads & history") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <section v-for="release in releases" :key="release.id" class="card">
    <h2>v{{ release.version }}</h2>
    <article
      v-for="asset in release.assets"
      :key="asset.id"
      class="download-options"
    >
      <h3>{{ asset.file_name }}</h3>
      <p>{{ asset.platform }} · {{ asset.architecture }}</p>
      <div class="actions">
        <SiteLink
          v-for="source in asset.sources"
          :key="source.id"
          class="button"
          :href="`/api/v1/downloads/account/assets/${asset.id}/sources/${source.id}`"
          >{{ source.provider_name }} ↓</SiteLink
        >
      </div>
    </article>
  </section>
  <SiteLink :href="en ? '/en/downloads' : '/downloads'" class="button"
    >{{ t("全部客户端下载", "All client downloads") }} ↗</SiteLink
  >
  <h2>{{ t("我的下载记录", "My download history") }}</h2>
  <WorkspaceApiTable endpoint="/api/v1/downloads/history" :columns="columns" />
</template>
