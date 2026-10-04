<script setup lang="ts">
const { t } = useLocale();
const list = useTemplateRef("list");
</script>
<template>
  <WorkspaceFrame admin
    ><h1>{{ t("下载管理", "Download management") }}</h1>
    <AdminNewDownload @done="list?.reload()" /><AdminAssetManager ref="list"
  /></WorkspaceFrame>
</template>
