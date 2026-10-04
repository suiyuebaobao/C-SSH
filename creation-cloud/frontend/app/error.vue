<script setup lang="ts">
import type { NuxtError } from "#app";
defineProps<{ error: NuxtError }>();
useHead({
  title: "C-SSH",
  meta: [{ name: "robots", content: "noindex, nofollow" }],
});
</script>
<template>
  <main class="container error-page">
    <a href="/" class="brand">C-SSH</a>
    <p class="eyebrow">{{ error.statusCode }}</p>
    <h1>
      {{
        error.statusCode === 404
          ? "页面不存在 / Page not found"
          : "页面暂不可用 / Page unavailable"
      }}
    </h1>
    <p>请稍后再试。Please try again later.</p>
    <a class="button" href="/">首页 / Home</a>
  </main>
</template>
