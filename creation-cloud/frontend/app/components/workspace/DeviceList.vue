<script setup lang="ts">
const props = defineProps<{ admin?: boolean }>();
const { t, en } = useLocale();
const api = useCloudApi();
const accountInput = ref("");
const account = ref("");
const deviceEndpoint = computed(
  () =>
    prefix.value +
    (account.value ? `?account_id=${encodeURIComponent(account.value)}` : ""),
);
const selected = ref<{ id: string; name: string } | null>(null);
const error = ref("");
const busy = ref(false);
const devices = useTemplateRef("devices");
const sessions = useTemplateRef("sessions");
const prefix = computed(() =>
  props.admin ? "/api/v1/admin/devices" : "/api/v1/devices",
);
const sessionEndpoint = computed(() =>
  props.admin ? "/api/v1/admin/sessions" : "/api/v1/devices/sessions",
);
const deviceColumns = computed(() => [
  { key: "name", label: t("名称", "Name") },
  { key: "platform", label: t("平台", "Platform") },
  { key: "last_seen_at", label: t("最近活动", "Last seen") },
  { key: "revoked_at", label: t("撤销时间", "Revoked") },
]);
const sessionColumns = computed(() => [
  { key: "device_name", label: t("设备", "Device") },
  { key: "status", label: t("状态", "Status") },
  { key: "is_current", label: t("当前会话", "Current") },
  { key: "last_seen_at", label: t("最近活动", "Last seen") },
  { key: "details", label: t("详情", "Details") },
]);
async function rename() {
  if (!selected.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api(`${prefix.value}/${selected.value.id}`, "PATCH", {
      name: selected.value.name,
    });
    selected.value = null;
    await devices.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function refresh() {
  await devices.value?.load();
  await sessions.value?.load();
}
</script>
<template>
  <h1>{{ t("设备与会话", "Devices & sessions") }}</h1>
  <h2>{{ t("已登记设备", "Registered devices") }}</h2>
  <form
    v-if="admin"
    class="filter-bar"
    @submit.prevent="account = accountInput.trim()"
  >
    <label
      >{{ t("按账号筛选", "Filter by account")
      }}<input v-model="accountInput" placeholder="Account ID" /></label
    ><button class="button">{{ t("筛选", "Filter") }}</button>
  </form>
  <WorkspaceApiTable
    ref="devices"
    :endpoint="deviceEndpoint"
    :columns="deviceColumns"
    ><template #actions="{ row }"
      ><button
        v-if="!admin"
        class="button"
        @click="selected = { id: String(row.id), name: String(row.name) }"
      >
        {{ t("重命名", "Rename") }}</button
      ><WorkspaceConfirmAction
        v-if="!row.revoked_at"
        :label="t('撤销设备', 'Revoke device')"
        :message="
          t(
            '该设备的全部会话将立即失效。其他设备和云端密文保持不变。',
            'All sessions on this device will be revoked. Other devices and encrypted data remain unchanged.',
          )
        "
        :endpoint="`${prefix}/${row.id}${admin ? '/revoke' : ''}`"
        :method="admin ? 'POST' : 'DELETE'"
        @done="refresh" /></template
  ></WorkspaceApiTable>
  <h2>{{ t("登录会话", "Sign-in sessions") }}</h2>
  <WorkspaceApiTable
    ref="sessions"
    :endpoint="sessionEndpoint"
    :columns="sessionColumns"
    ><template #details="{ row }"
      ><details>
        <summary>{{ t("展开", "Details") }}</summary>
        <dl class="detail-list">
          <template
            v-for="key in [
              'session_id',
              'account_label',
              'last_login_ip',
              'user_agent',
              'client_version',
              'device_fingerprint',
              'created_at',
              'idle_expires_at',
              'absolute_expires_at',
              'revoked_at',
            ]"
            :key="key"
            ><dt>{{ key }}</dt>
            <dd>{{ row[key] || "—" }}</dd></template
          >
        </dl>
      </details></template
    ><template #actions="{ row }"
      ><WorkspaceConfirmAction
        :label="t('撤销会话', 'Revoke session')"
        :message="
          t(
            '目标会话将立即失效；撤销当前会话后需重新登录。',
            'This session will be revoked. Revoking your current session signs you out.',
          )
        "
        :endpoint="`${sessionEndpoint}/${row.session_id}`"
        @done="refresh" /></template></WorkspaceApiTable
  ><Teleport to="body"
    ><div v-if="selected" class="modal-overlay">
      <section
        class="modal card"
        role="dialog"
        aria-modal="true"
        :aria-label="t('重命名设备', 'Rename device')"
      >
        <form @submit.prevent="rename">
          <label
            >{{ t("名称", "Name")
            }}<input v-model="selected.name" required maxlength="100"
          /></label>
          <p v-if="error" role="alert" class="form-error">{{ error }}</p>
          <button
            class="button"
            type="button"
            :disabled="busy"
            @click="selected = null"
          >
            {{ t("取消", "Cancel") }}</button
          ><button class="button primary" :disabled="busy">
            {{ t("保存", "Save") }}
          </button>
        </form>
      </section>
    </div></Teleport
  >
</template>
