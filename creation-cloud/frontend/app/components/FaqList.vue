<script setup lang="ts">
import type { Faq } from "~/types/site";
defineProps<{ items: Faq[] }>();
</script>
<template>
  <div class="faq-list">
    <details v-for="item in items" :key="item.question">
      <summary>{{ item.question }}<span aria-hidden="true">+</span></summary>
      <p>{{ item.answer }}</p>
    </details>
  </div>
</template>
