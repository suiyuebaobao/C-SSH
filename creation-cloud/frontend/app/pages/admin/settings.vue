<script setup lang="ts">
const { t } = useLocale();
const columns = computed(() => [
  { key: "task", label: t("任务", "Task") },
  { key: "last_observation", label: t("状态", "Status") },
  { key: "last_success_at", label: t("最近成功", "Last success") },
  { key: "consecutive_failures", label: t("连续失败", "Consecutive failures") },
]);
</script>
<template>
  <WorkspaceFrame admin
    ><h1>{{ t("设置", "Settings") }}</h1>
    <AdminAuthSettings />
    <h2>{{ t("维护状态", "Maintenance status") }}</h2>
    <WorkspaceApiTable
      endpoint="/api/v1/admin/maintenance"
      :columns="columns"
      array
  /></WorkspaceFrame>
</template>
