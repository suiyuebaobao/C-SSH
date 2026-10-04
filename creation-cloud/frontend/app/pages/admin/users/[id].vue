<script setup lang="ts">
const route = useRoute();
const id = String(route.params.id);
if (!/^[0-9a-f-]{36}$/i.test(id)) throw createError({ statusCode: 404 });
</script>
<template>
  <WorkspaceFrame admin><AdminUserDetail :id="id" /></WorkspaceFrame>
</template>
