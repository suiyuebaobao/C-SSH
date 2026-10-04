<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const file = ref<File | null>(null),
  altZh = ref(""),
  altEn = ref("");
const busy = ref(false),
  error = ref("");
const selected = ref<Record<string, unknown> | null>(null);
const table = useTemplateRef("table");
const columns = computed(() => [
  { key: "alt_zh", label: t("中文说明", "Chinese description") },
  { key: "alt_en", label: t("英文说明", "English description") },
  { key: "state", label: t("状态", "Status") },
  { key: "byte_size", label: t("大小（字节）", "Size (bytes)") },
]);
async function upload() {
  if (!file.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    const body = new FormData();
    body.set("file", file.value);
    body.set("alt_zh", altZh.value);
    body.set("alt_en", altEn.value);
    await api("/api/v1/admin/site-media", "POST", body);
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
function edit(row: Record<string, unknown>) {
  selected.value = { ...row };
  altZh.value = String(row.alt_zh);
  altEn.value = String(row.alt_en);
}
async function save() {
  if (!selected.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api(`/api/v1/admin/site-media/${selected.value.id}`, "PATCH", {
      alt_zh: altZh.value,
      alt_en: altEn.value,
    });
    selected.value = null;
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="card">
    <h2>{{ t("网站图片", "Site media") }}</h2>
    <form @submit.prevent="selected ? save() : upload()">
      <label v-if="!selected"
        >{{ t("PNG 图片（不超过 2 MiB）", "PNG image (up to 2 MiB)")
        }}<input
          type="file"
          accept="image/png"
          required
          @change="
            file = ($event.target as HTMLInputElement).files?.[0] || null
          " /></label
      ><label
        >{{ t("中文说明", "Chinese description")
        }}<input v-model="altZh" required /></label
      ><label
        >{{ t("英文说明", "English description")
        }}<input v-model="altEn" required
      /></label>
      <p v-if="error" role="alert" class="form-error">{{ error }}</p>
      <button class="button primary" :disabled="busy">
        {{
          selected
            ? t("保存说明", "Save description")
            : t("上传草稿", "Upload draft")
        }}</button
      ><button
        v-if="selected"
        class="button"
        type="button"
        @click="selected = null"
      >
        {{ t("取消编辑", "Cancel editing") }}
      </button>
    </form>
  </section>
  <WorkspaceApiTable
    ref="table"
    endpoint="/api/v1/admin/site-media"
    :columns="columns"
    array
    ><template #actions="{ row, reload }"
      ><button class="button" @click="edit(row)">
        {{ t("编辑说明", "Edit description") }}</button
      ><WorkspaceConfirmAction
        v-if="row.state === 'draft'"
        :label="t('发布', 'Publish')"
        :message="t('确认发布这张图片？', 'Publish this image?')"
        :endpoint="`/api/v1/admin/site-media/${row.id}/publish`"
        method="POST"
        @done="reload" /><WorkspaceConfirmAction
        v-if="row.state === 'published'"
        :label="t('撤下', 'Revoke')"
        :message="
          t(
            '撤下后公开页面将无法显示这张图片。',
            'This image will no longer be available publicly.',
          )
        "
        :endpoint="`/api/v1/admin/site-media/${row.id}/revoke`"
        method="POST"
        @done="reload" /><WorkspaceConfirmAction
        v-if="row.state === 'draft'"
        :label="t('删除', 'Delete')"
        :message="t('永久删除这张图片？', 'Permanently delete this image?')"
        :endpoint="`/api/v1/admin/site-media/${row.id}`"
        @done="reload" /></template
  ></WorkspaceApiTable>
</template>
