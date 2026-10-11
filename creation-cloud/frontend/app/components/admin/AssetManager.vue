<script setup lang="ts">
import type { AdminAsset, AdminSource, AdminRelease } from "~/types/releases";
const { t, en } = useLocale();
const api = useCloudApi();
const selected = ref<AdminAsset | null>(null);
const release = ref<AdminRelease | null>(null);
const sources = ref<AdminSource[]>([]);
const signature = ref<File | null>(null);
const error = ref(""),
  notice = ref(""),
  busy = ref(false);
const table = useTemplateRef("table");
const sourceName = ref("");
const sourceUrl = ref("");
const sourceMutable = computed(
  () =>
    release.value !== null &&
    !["hidden", "revoked"].includes(release.value.status),
);
const mutable = computed(() =>
  ["draft", "validating"].includes(release.value?.status ?? ""),
);
const columns = computed(() => [
  { key: "file_name", label: t("文件", "File") },
  { key: "platform", label: t("平台", "Platform") },
  { key: "architecture", label: t("架构", "Architecture") },
  { key: "package_kind", label: t("格式", "Package") },
  { key: "byte_size", label: t("字节数", "Size (bytes)") },
]);
async function open(row: Record<string, unknown>) {
  error.value = "";
  notice.value = "";
  selected.value = row as unknown as AdminAsset;
  try {
    const [items, parent] = await Promise.all([
      api<AdminSource[]>(`/api/v1/admin/downloads/assets/${row.id}/sources`),
      api<AdminRelease>(`/api/v1/admin/releases/${row.release_id}`),
    ]);
    sources.value = items;
    release.value = parent;
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
async function reloadSources() {
  if (!selected.value) return;
  try {
    sources.value = await api(
      `/api/v1/admin/downloads/assets/${selected.value.id}/sources`,
    );
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
async function addSource() {
  if (!selected.value || busy.value || !sourceMutable.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api(
      `/api/v1/admin/downloads/assets/${selected.value.id}/sources`,
      "POST",
      {
        source_kind: "external",
        provider_name: sourceName.value,
        external_url: sourceUrl.value,
        local_path: null,
        sort_order: 0,
        enabled: true,
      },
    );
    sourceName.value = "";
    sourceUrl.value = "";
    await reloadSources();
    notice.value = t("下载来源已添加。", "Download source added.");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function saveSource(source: AdminSource) {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    await api(`/api/v1/admin/downloads/sources/${source.id}`, "PATCH", {
      enabled: source.enabled,
      sort_order: source.sort_order,
    });
    notice.value = t("来源设置已保存。", "Source settings saved.");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function uploadSignature() {
  if (!selected.value || !signature.value || busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    const body = new FormData();
    body.set("signature", signature.value);
    await api(
      `/api/v1/admin/downloads/assets/${selected.value.id}/updater-signature`,
      "POST",
      body,
    );
    notice.value = t("签名已保存。", "Signature saved.");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
async function removed() {
  selected.value = null;
  await table.value?.load();
}
defineExpose({ reload: () => table.value?.load() });
</script>
<template>
  <h2>{{ t("已上传客户端", "Uploaded clients") }}</h2>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
  <WorkspaceApiTable
    ref="table"
    endpoint="/api/v1/admin/releases/assets"
    :columns="columns"
    ><template #actions="{ row }"
      ><button class="button" @click="open(row)">
        {{ t("来源与校验", "Sources & integrity") }}
      </button></template
    ></WorkspaceApiTable
  >
  <section v-if="selected" class="card">
    <h2>{{ selected.file_name }}</h2>
    <p>{{ release?.version }} · {{ t("修订", "Revision") }} {{ release?.revision }} · {{ release?.status }}</p>
    <details>
      <summary>{{ t("文件校验", "File integrity") }}</summary>
      <pre>
SHA256: {{ selected.sha256 }}
{{ t("字节数", "Bytes") }}: {{ selected.byte_size }}</pre
      >
    </details>
    <h3>{{ t("下载来源", "Download sources") }}</h3>
    <form v-if="sourceMutable" @submit.prevent="addSource">
      <label
        >{{ t("来源名称", "Source name") }}<input v-model="sourceName" required
      /></label>
      <label
        >{{ t("HTTPS 下载地址", "HTTPS download URL")
        }}<input
          v-model="sourceUrl"
          type="url"
          required
          placeholder="https://example.com/client.zip"
      /></label>
      <button class="button" :disabled="busy">
        {{ t("添加外链来源", "Add external source") }}
      </button>
    </form>
    <form
      v-for="source in sources"
      :key="source.id"
      @submit.prevent="saveSource(source)"
    >
      <h4>{{ source.provider_name }}</h4>
      <label
        >{{ t("顺序", "Order")
        }}<input
          v-model.number="source.sort_order"
          type="number"
          required /></label
      ><label
        ><input v-model="source.enabled" type="checkbox" />{{
          t("启用", "Enabled")
        }}</label
      ><button class="button" :disabled="busy">{{ t("保存", "Save") }}</button
      ><WorkspaceConfirmAction
        :label="t('删除来源', 'Delete source')"
        :message="t('确认删除此下载来源？', 'Delete this download source?')"
        :endpoint="`/api/v1/admin/downloads/sources/${source.id}`"
        @done="reloadSources"
      />
    </form>
    <form
      v-if="mutable && selected.platform === 'windows'"
      @submit.prevent="uploadSignature"
    >
      <label
        >{{ t("更新签名文件", "Updater signature file")
        }}<input
          type="file"
          accept=".sig"
          required
          @change="
            signature = ($event.target as HTMLInputElement).files?.[0] || null
          " /></label
      ><button class="button" :disabled="busy">
        {{ t("上传签名", "Upload signature") }}
      </button>
    </form>
    <WorkspaceConfirmAction
      v-if="mutable"
      :label="t('删除资产', 'Delete asset')"
      :message="t('确认删除这项未发布资产？', 'Delete this unpublished asset?')"
      :endpoint="`/api/v1/admin/releases/assets/${selected.id}`"
      @done="removed"
    />
  </section>
</template>
