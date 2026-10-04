<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const selected = ref<Record<string, unknown> | null>(null);
const error = ref("");
const columns = computed(() => [
  { key: "action", label: t("操作", "Action") },
  { key: "resource_kind", label: t("资源", "Resource") },
  { key: "outcome", label: t("结果", "Outcome") },
  { key: "created_at", label: t("时间", "Time") },
]);
async function open(id: unknown) {
  try {
    selected.value = await api(`/api/v1/admin/audit-events/${id}`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
</script>
<template>
  <h1>{{ t("审计记录", "Audit trail") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <WorkspaceApiTable endpoint="/api/v1/admin/audit-events" :columns="columns"
    ><template #actions="{ row }"
      ><button class="button" @click="open(row.id)">
        {{ t("详情", "Details") }}
      </button></template
    ></WorkspaceApiTable
  >
  <section v-if="selected" class="card">
    <h2>{{ selected.action }}</h2>
    <dl class="detail-list">
      <dt>{{ t("请求编号", "Request ID") }}</dt>
      <dd>{{ selected.request_id }}</dd>
      <dt>{{ t("资源编号", "Resource ID") }}</dt>
      <dd>{{ selected.resource_id }}</dd>
    </dl>
    <pre>{{ JSON.stringify(selected.details, null, 2) }}</pre>
  </section>
</template>
