<script setup lang="ts">
import type { Page } from "~/types/site";
defineProps<{ page: Page }>();
</script>
<template>
  <header class="page-heading">
    <p class="eyebrow">{{ page.eyebrow }}</p>
    <h1>{{ page.heading }}</h1>
    <p class="lead">{{ page.lead }}</p>
    <div v-if="page.actions.length" class="actions">
      <SiteLink
        v-for="(item, index) in page.actions"
        :key="item.href"
        :href="item.href"
        :class="['button', { primary: index === 0 }]"
        >{{ item.label }} <span aria-hidden="true">↗</span></SiteLink
      >
    </div>
  </header>
</template>
