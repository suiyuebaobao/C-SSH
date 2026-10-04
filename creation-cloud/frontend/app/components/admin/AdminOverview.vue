<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const data = ref<Record<string, Record<string, unknown>> | null>(null);
const error = ref("");
onMounted(async () => {
  try {
    data.value = await api("/api/v1/admin/overview");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
const metrics = computed(() =>
  data.value
    ? [
        [t("用户", "Users"), data.value.users?.total_users],
        [t("设备", "Devices"), data.value.devices?.total_devices],
        [
          t("已发布版本", "Published releases"),
          data.value.releases?.published_releases,
        ],
        [t("审计事件", "Audit events"), data.value.audit?.total_events],
      ]
    : [],
);
const probes = computed(() => [
  ["database", t("数据库", "Database")],
  ["downloads", t("下载存储", "Download storage")],
  ["site_content", t("网站内容", "Site content")],
  ["site_media", t("媒体存储", "Media storage")],
]);
</script>
<template>
  <h1>{{ t("管理总览", "Administration overview") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <div class="stats-grid">
    <article v-for="item in metrics" :key="String(item[0])" class="card">
      <span>{{ item[0] }}</span
      ><strong>{{ item[1] }}</strong>
    </article>
  </div>
  <section v-if="data" class="card">
    <h2>{{ t("服务状态", "Service health") }}</h2>
    <dl class="detail-list">
      <template v-for="probe in probes" :key="probe[0]"
        ><dt>{{ probe[1] }}</dt>
        <dd>
          {{
            data.health?.[probe[0]!]
              ? t("正常", "Ready")
              : t("异常", "Unavailable")
          }}
        </dd></template
      >
    </dl>
  </section>
</template>
