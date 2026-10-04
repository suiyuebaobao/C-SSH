<script setup lang="ts">
import { seo } from "~/config/resources";
</script>
<template>
  <WorkspaceFrame admin
    ><AdminResourceManager :config="seo" deletable
  /></WorkspaceFrame>
</template>
