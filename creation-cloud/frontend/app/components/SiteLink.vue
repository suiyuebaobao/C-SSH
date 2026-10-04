<script setup lang="ts">
defineProps<{ href: string }>();
</script>
<template>
  <!-- 未迁移的认证／后台保持完整文档导航，不能由Nuxt catch-all截获。 -->
  <a :href="href"><slot /></a>
</template>
