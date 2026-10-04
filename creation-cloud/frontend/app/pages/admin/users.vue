<script setup lang="ts">
import { users } from "~/config/resources";
const { t, en } = useLocale();
</script>
<template>
  <WorkspaceFrame admin
    ><AdminResourceManager :config="users" deletable
      ><template #actions="{ row }"
        ><NuxtLink
          class="button"
          :to="`/admin/users/${row.id}${en ? '?lang=en' : ''}`"
          >{{ t("详情", "Details") }}</NuxtLink
        ></template
      ></AdminResourceManager
    ></WorkspaceFrame
  >
</template>
