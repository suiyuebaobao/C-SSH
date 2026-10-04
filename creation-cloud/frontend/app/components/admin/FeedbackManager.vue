<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const selected = ref<Record<string, unknown> | null>(null);
const status = ref("triaged");
const reason = ref("");
const busy = ref(false);
const error = ref("");
const table = useTemplateRef("table");
const statusFilter = ref("");
const endpoint = computed(
  () =>
    "/api/v1/admin/feedback" +
    (statusFilter.value ? `?status=${statusFilter.value}` : ""),
);
const columns = computed(() => [
  { key: "id", label: t("反馈编号", "Feedback ID") },
  { key: "category", label: t("类别", "Category") },
  { key: "platform", label: t("平台", "Platform") },
  { key: "status", label: t("状态", "Status") },
  { key: "redacted", label: t("已脱敏", "Redacted") },
]);
async function open(id: unknown) {
  error.value = "";
  reason.value = "";
  try {
    selected.value = await api(`/api/v1/admin/feedback/${id}`);
    status.value = String(selected.value?.status);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
async function update() {
  if (!selected.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    selected.value = await api(
      `/api/v1/admin/feedback/${selected.value.id}/status`,
      "PATCH",
      {
        status: status.value,
        reason: reason.value,
        expected_version: selected.value.version,
      },
    );
    reason.value = "";
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function redacted() {
  if (selected.value) await open(selected.value.id);
  await table.value?.load();
}
</script>
<template>
  <h1>{{ t("用户反馈", "User feedback") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <label
    >{{ t("筛选状态", "Filter status")
    }}<select v-model="statusFilter">
      <option value="">{{ t("全部", "All") }}</option>
      <option
        v-for="value in ['new', 'triaged', 'in_progress', 'resolved', 'closed']"
        :key="value"
        :value="value"
      >
        {{ value }}
      </option>
    </select></label
  >
  <WorkspaceApiTable ref="table" :endpoint="endpoint" :columns="columns"
    ><template #actions="{ row }"
      ><button class="button" @click="open(row.id)">
        {{ t("查看详情", "View details") }}
      </button></template
    ></WorkspaceApiTable
  >
  <section v-if="selected" class="card">
    <h2>{{ selected.title }}</h2>
    <p class="plain-text">{{ selected.description }}</p>
    <p>{{ selected.redaction_reason }}</p>
    <form @submit.prevent="update">
      <label
        >{{ t("处理状态", "Status")
        }}<select v-model="status">
          <option
            v-for="value in [
              'new',
              'triaged',
              'in_progress',
              'resolved',
              'closed',
            ]"
            :key="value"
            :value="value"
          >
            {{ value }}
          </option>
        </select></label
      ><label
        >{{ t("处理原因", "Reason") }}<input v-model="reason" required /></label
      ><button class="button primary" :disabled="busy">
        {{ t("保存状态", "Save status") }}
      </button>
    </form>
    <WorkspaceConfirmAction
      v-if="!selected.redacted && reason.trim()"
      :label="t('永久脱敏正文', 'Permanently redact content')"
      :message="
        t(
          '此操作永久移除原始标题和正文，不能恢复。',
          'Permanently removes the original title and description. This cannot be undone.',
        )
      "
      :endpoint="`/api/v1/admin/feedback/${selected.id}/redact`"
      method="POST"
      :body="{ expected_version: selected.version, reason }"
      @done="redacted"
    />
  </section>
</template>
