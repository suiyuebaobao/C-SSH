<script setup lang="ts">
import { releases } from "~/config/resources";
const { t, en } = useLocale();
const route = useRoute();
const tab = ref(
  route.path.endsWith("/upload")
    ? "upload"
    : route.path.endsWith("/uploaded")
      ? "files"
      : "versions",
);
const tabs = computed(() => [
  ["versions", t("版本列表", "Versions")],
  ["policy", t("更新设置", "Update policy")],
  ["upload", t("上传客户端", "Upload client")],
  ["files", t("已上传", "Uploaded")],
]);
</script>
<template>
  <div class="filter-bar">
    <button
      v-for="item in tabs"
      :key="item[0]"
      :class="['button', { primary: tab === item[0] }]"
      @click="tab = item[0]!"
    >
      {{ item[1] }}
    </button>
  </div>
  <AdminResourceManager
    v-if="tab === 'versions'"
    :config="releases"
    deletable
    readonly-published
    ><template #actions="{ row, reload }"
      ><WorkspaceConfirmAction
        v-if="row.status === 'validating'"
        :label="t('正式发布', 'Publish release')"
        :message="
          t(
            '确认发布此版本？发布后版本和资产不得覆盖，请先完成校验。',
            'Publish this release? Released files are immutable. Complete validation first.',
          )
        "
        :endpoint="`/api/v1/admin/releases/${row.id}`"
        method="PATCH"
        :body="{ status: 'published' }"
        @done="reload"
      />
      <WorkspaceConfirmAction
        v-if="row.status === 'published'"
        :label="t('新建修订', 'Create revision')"
        :message="t('为此版本新建修订草稿？原发布与旧文件保持不变，新文件需重新上传并通过校验。', 'Create a new revision draft? The original release and files remain unchanged. Upload and validate the new files separately.')"
        :endpoint="`/api/v1/admin/releases/${row.id}/revisions`"
        method="POST"
        :body="{ expected_updated_at: row.updated_at, expected_latest_revision: row.latest_revision }"
        @done="reload"
      />
      <WorkspaceConfirmAction
        v-if="row.status === 'published'"
        :label="t('隐藏版本', 'Hide release')"
        :message="
          t(
            '隐藏后不再展示该版本，不能恢复为公开发布；原文件保持不变。',
            'This release will no longer be listed and cannot be republished. Original files remain unchanged.',
          )
        "
        :endpoint="`/api/v1/admin/releases/${row.id}`"
        method="PATCH"
        :body="{ status: 'hidden' }"
        @done="reload"
      />
      <WorkspaceConfirmAction
        v-if="row.status === 'published'"
        :label="t('撤销发布', 'Revoke release')"
        :message="
          t(
            '确认撤销此版本？此状态不能恢复。',
            'Revoke this release? This state cannot be reversed.',
          )
        "
        :endpoint="`/api/v1/admin/releases/${row.id}`"
        method="PATCH"
        :body="{ status: 'revoked' }"
        @done="reload"
      />
      <NuxtLink
        :to="`/admin/assets?release_id=${row.id}${en ? '&lang=en' : ''}`"
        class="button"
        >{{ t("下载管理", "Downloads") }}</NuxtLink
      ></template
    ></AdminResourceManager
  ><AdminUpdatePolicy v-else-if="tab === 'policy'" /><AdminNewDownload
    v-else-if="tab === 'upload'"
    @done="tab = 'files'"
  /><AdminAssetManager v-else />
</template>
