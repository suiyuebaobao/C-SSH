<script setup lang="ts">
import type { Section } from "~/types/site";
defineProps<{ sections: Section[] }>();
</script>
<template>
  <section
    v-for="section in sections"
    :id="section.anchor"
    :key="section.anchor"
    class="content-section"
  >
    <div class="section-heading">
      <h2>{{ section.title }}</h2>
      <p>{{ section.lead }}</p>
    </div>
    <div class="card-grid">
      <article v-for="(item, index) in section.items" :key="index" class="card">
        <span class="eyebrow">{{ item.badge }}</span>
        <h3>{{ item.title }}</h3>
        <p>{{ item.body }}</p>
        <small v-if="item.meta">{{ item.meta }}</small>
      </article>
    </div>
  </section>
</template>
