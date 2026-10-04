<script setup lang="ts">
const props = defineProps<{ mode: string }>();
const { t } = useLocale();
const hostColumns = computed(() => [
  { key: "id", label: t("主机记录标识", "Host record ID") },
  { key: "revision", label: t("版本", "Revision") },
  { key: "source_device_id", label: t("来源设备", "Source device") },
  { key: "deleted", label: t("已删除", "Deleted") },
  { key: "secret_present", label: t("存在密文", "Encrypted data present") },
  { key: "metadata_encrypted", label: t("资料已加密", "Metadata encrypted") },
  {
    key: "host_metadata_migration_required",
    label: t("需要加密迁移", "Encryption migration required"),
  },
  { key: "updated_at", label: t("更新时间", "Updated") },
]);
const modelColumns = computed(() => [
  { key: "name", label: t("模型", "Model") },
  { key: "provider", label: t("厂商", "Provider") },
  { key: "context_length", label: t("上下文长度", "Context length") },
  { key: "interfaces", label: t("接口", "Interfaces") },
]);
const title = computed(() =>
  props.mode === "hosts"
    ? t("云端密文", "Encrypted cloud data")
    : props.mode === "models"
      ? t("模型目录", "Model directory")
      : t("下载记录", "Download history"),
);
</script>
<template>
  <h1>{{ title }}</h1>
  <template v-if="mode === 'hosts'"
    ><p>
      {{
        t(
          "这里只显示不透明记录身份与加密状态。请在客户端解锁、预览并手动同步。",
          "Only opaque record IDs and encryption status are shown. Unlock, preview and manually sync in the client.",
        )
      }}
    </p>
    <WorkspaceApiTable
      endpoint="/api/v1/hosts"
      :columns="hostColumns" /></template
  ><WorkspaceApiTable
    v-else-if="mode === 'models'"
    endpoint="/api/v1/models"
    :columns="modelColumns"
    ><template #interfaces="{ row }"
      ><ul>
        <li
          v-for="item in row.interfaces as {
            api_format: string;
            model_name: string;
            base_url: string;
          }[]"
          :key="item.api_format"
        >
          <strong>{{ item.api_format }}</strong>
          <p>{{ item.model_name }}</p>
          <small>{{ item.base_url }}</small>
        </li>
      </ul></template
    ></WorkspaceApiTable
  >
</template>
