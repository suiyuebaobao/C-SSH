<script setup lang="ts">
const props = defineProps<{ id: string }>();
const { t, en } = useLocale();
const api = useCloudApi();
const user = ref<Record<string, unknown> | null>(null);
const error = ref("");
const prefix = computed(() => `/api/v1/admin/users/${props.id}`);
onMounted(async () => {
  try {
    user.value = await api(`${prefix.value}/profile`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
const hosts = computed(() => [
  { key: "id", label: t("加密主机编号", "Encrypted host ID") },
  { key: "revision", label: t("版本", "Revision") },
  { key: "secret_present", label: t("存在密文", "Encrypted data") },
  { key: "metadata_encrypted", label: t("资料已加密", "Metadata encrypted") },
  { key: "deleted", label: t("已删除", "Deleted") },
]);
const sync = computed(() => [
  { key: "device_name", label: t("设备", "Device") },
  { key: "direction", label: t("方向", "Direction") },
  { key: "outcome", label: t("结果", "Outcome") },
  { key: "changed_count", label: t("记录数", "Records") },
  { key: "occurred_at", label: t("时间", "Time") },
]);
const sessions = computed(() => [
  { key: "device_name", label: t("设备", "Device") },
  { key: "status", label: t("状态", "Status") },
  { key: "is_current", label: t("当前会话", "Current session") },
  { key: "last_login_ip", label: t("最近登录 IP", "Last sign-in IP") },
  { key: "last_seen_at", label: t("最近活动", "Last seen") },
]);
</script>
<template>
  <h1>{{ t("用户详情", "User details") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <section v-if="user" class="card">
    <h2>{{ user.display_name }}</h2>
    <dl class="detail-list">
      <dt>{{ t("邮箱", "Email") }}</dt>
      <dd>{{ user.email }}</dd>
      <dt>{{ t("角色", "Role") }}</dt>
      <dd>{{ user.role }}</dd>
      <dt>{{ t("状态", "Status") }}</dt>
      <dd>{{ user.status }}</dd>
    </dl>
  </section>
  <h2>{{ t("加密主机", "Encrypted hosts") }}</h2>
  <p>
    {{
      t(
        "删除只处理云端记录，不连接或修改远程主机。",
        "Deletion affects cloud records only. Remote hosts are not contacted or changed.",
      )
    }}
  </p>
  <WorkspaceApiTable :endpoint="`${prefix}/hosts`" :columns="hosts"
    ><template #actions="{ row, reload }"
      ><WorkspaceConfirmAction
        v-if="!row.deleted"
        :label="t('删除云端主机', 'Delete cloud host')"
        :message="
          t(
            '确认删除该用户的云端主机记录？',
            'Delete this user’s cloud host record?',
          )
        "
        :endpoint="`${prefix}/hosts/${row.id}`"
        @done="reload" /></template
  ></WorkspaceApiTable>
  <h2>{{ t("同步记录", "Sync records") }}</h2>
  <WorkspaceApiTable :endpoint="`${prefix}/sync-records`" :columns="sync"
    ><template #actions="{ row, reload }"
      ><WorkspaceConfirmAction
        :label="t('删除记录', 'Delete record')"
        :message="
          t('确认永久删除此同步记录？', 'Permanently delete this sync record?')
        "
        :endpoint="`${prefix}/sync-records/${encodeURIComponent(String(row.record_id))}`"
        @done="reload" /></template
  ></WorkspaceApiTable>
  <h2>{{ t("登录会话", "Sign-in sessions") }}</h2>
  <WorkspaceApiTable :endpoint="`${prefix}/sessions`" :columns="sessions"
    ><template #actions="{ row, reload }"
      ><WorkspaceConfirmAction
        :label="t('撤销', 'Revoke')"
        :message="t('确认撤销该用户会话？', 'Revoke this user session?')"
        :endpoint="`/api/v1/admin/sessions/${row.session_id}`"
        @done="reload" /></template
  ></WorkspaceApiTable>
</template>
