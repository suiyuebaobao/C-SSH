<script setup lang="ts">
import type { ContentNode } from "~/types/forms";
const { t, en } = useLocale();
const api = useCloudApi();
const key = ref("home"),
  locale = ref("zh-CN"),
  error = ref("");
const busy = ref(false),
  preview = ref(false);
interface ContentRecord {
  id: string;
  state: string;
  revision: number;
  content: { document_key: string; content: Record<string, ContentNode> };
}
const selected = ref<ContentRecord | null>(null);
const table = useTemplateRef("table");
const endpoint = computed(
  () =>
    `/api/v1/admin/site-content?document_key=${key.value}&locale=${locale.value}`,
);
const columns = computed(() => [
  { key: "document_key", label: t("页面", "Page") },
  { key: "locale", label: t("语言", "Language") },
  { key: "state", label: t("状态", "Status") },
  { key: "revision", label: t("修订", "Revision") },
  { key: "updated_at", label: t("更新时间", "Updated") },
]);
async function create() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    selected.value = await api("/api/v1/admin/site-content", "POST", {
      document_key: key.value,
      locale: locale.value,
    });
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function open(id: unknown) {
  error.value = "";
  preview.value = false;
  try {
    selected.value = await api(`/api/v1/admin/site-content/${id}`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
async function save() {
  if (!selected.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    selected.value = await api(
      `/api/v1/admin/site-content/${selected.value.id}`,
      "PATCH",
      {
        expected_revision: selected.value.revision,
        content: selected.value.content,
      },
    );
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function refresh() {
  selected.value = null;
  await table.value?.load();
}
</script>
<template>
  <h1>{{ t("网站内容", "Site content") }}</h1>
  <div class="filter-bar">
    <select v-model="key" :aria-label="t('页面', 'Page')">
      <option value="home">{{ t("首页", "Home") }}</option>
      <option value="site_shell">
        {{ t("导航与页脚", "Navigation & footer") }}
      </option></select
    ><select v-model="locale" :aria-label="t('语言', 'Language')">
      <option value="zh-CN">中文</option>
      <option value="en">English</option></select
    ><button class="button primary" :disabled="busy" @click="create">
      {{ t("新建草稿", "New draft") }}
    </button>
  </div>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <WorkspaceApiTable ref="table" :endpoint="endpoint" :columns="columns" array
    ><template #actions="{ row, reload }"
      ><button class="button" @click="open(row.id)">
        {{ t("查看／编辑", "View / edit") }}</button
      ><WorkspaceConfirmAction
        :disabled="busy"
        v-if="row.state === 'draft'"
        :label="t('发布', 'Publish')"
        :message="t('确认发布这份已保存的草稿？', 'Publish this saved draft?')"
        :endpoint="`/api/v1/admin/site-content/${row.id}/publish`"
        method="POST"
        :body="{ expected_revision: row.revision }"
        @done="refresh" /><WorkspaceConfirmAction
        :disabled="busy"
        v-if="row.state === 'published'"
        :label="t('撤下', 'Revoke')"
        :message="
          t(
            '撤下后该公开内容将不可用，直至发布新的内容。',
            'The public content will be unavailable until a new revision is published.',
          )
        "
        :endpoint="`/api/v1/admin/site-content/${row.id}/revoke`"
        method="POST"
        :body="{ expected_revision: row.revision }"
        @done="refresh" /><WorkspaceConfirmAction
        :disabled="busy"
        v-if="row.state === 'draft'"
        :label="t('删除草稿', 'Delete draft')"
        :message="t('永久删除这份草稿？', 'Permanently delete this draft?')"
        :endpoint="`/api/v1/admin/site-content/${row.id}`"
        :body="{ expected_revision: row.revision }"
        @done="refresh" /><WorkspaceConfirmAction
        :disabled="busy"
        v-if="row.state === 'revoked'"
        :label="t('恢复此版本', 'Restore this revision')"
        :message="
          t(
            '使用此历史内容创建新的已发布修订？',
            'Create a new published revision from this historical content?',
          )
        "
        :endpoint="`/api/v1/admin/site-content/${row.id}/rollback`"
        method="POST"
        :body="{ expected_revision: row.revision }"
        @done="refresh" /></template
  ></WorkspaceApiTable>
  <section v-if="selected" class="card">
    <h2>{{ t("内容编辑", "Content editor") }}</h2>
    <p>
      {{
        t(
          "仅保存不会影响公开页面，确认预览后再发布。",
          "Saving does not change the public page. Review before publishing.",
        )
      }}
    </p>
    <button class="button" @click="preview = !preview">
      {{
        preview
          ? t("返回编辑", "Back to editor")
          : t("预览内容", "Preview content")
      }}</button
    ><AdminContentPreview v-if="preview" :payload="selected.content" />
    <form v-else @submit.prevent="save">
      <fieldset :disabled="selected.state !== 'draft' || busy">
        <AdminContentFields
          :value="selected.content.content"
          name="content"
          @change="
            (value) => {
              if (
                selected &&
                value &&
                typeof value === 'object' &&
                !Array.isArray(value)
              )
                selected.content.content = value;
            }
          "
        />
      </fieldset>
      <button
        v-if="selected.state === 'draft'"
        class="button primary"
        :disabled="busy"
      >
        {{ t("保存草稿", "Save draft") }}
      </button>
    </form>
  </section>
  <AdminMediaManager />
</template>
