<script setup lang="ts">
import type { ContactWidget } from "~/types/site";
defineProps<{ widget: ContactWidget }>();
const { t, en } = useLocale();
const opened = ref(false);
const loading = ref(false);
const error = ref("");
const media = ref<{
  content_url: string;
  alt_zh: string;
  alt_en: string;
} | null>(null);
async function open() {
  opened.value = true;
  loading.value = true;
  error.value = "";
  try {
    media.value = await $fetch("/api/v1/site-media/home-qr", {
      credentials: "omit",
      retry: 0,
    });
  } catch {
    error.value = t("联系图片暂不可用。", "Contact image unavailable.");
  } finally {
    loading.value = false;
  }
}
</script>
<template>
  <button class="button contact-trigger" @click="open">
    {{ widget.title }}</button
  ><Teleport to="body"
    ><div v-if="opened" class="modal-overlay">
      <section
        class="modal card"
        role="dialog"
        aria-modal="true"
        :aria-label="widget.title"
      >
        <h2>{{ widget.title }}</h2>
        <p>{{ widget.note }}</p>
        <p v-if="loading">{{ t("加载中…", "Loading…") }}</p>
        <p v-else-if="error" role="alert">{{ error }}</p>
        <img
          v-else-if="media"
          :src="media.content_url"
          :alt="en ? media.alt_en : media.alt_zh"
          width="240"
          height="240"
        />
        <p v-else>
          {{ widget.pending_label }}
        </p>
        <button class="button" @click="opened = false">
          {{ widget.close_label }}
        </button>
      </section>
    </div></Teleport
  >
</template>
