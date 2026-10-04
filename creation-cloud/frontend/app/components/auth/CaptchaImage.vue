<script setup lang="ts">
const props = defineProps<{ purpose: string }>();
const emit = defineEmits<{ challenge: [id: string] }>();
const { t } = useLocale();
const src = ref("");
const busy = ref(false);
const error = ref("");
async function refresh() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  emit("challenge", "");
  try {
    const response = await $fetch.raw<Blob>("/api/v1/auth/captcha", {
      query: { purpose: props.purpose, refresh: Date.now() },
      responseType: "blob",
      retry: 0,
    });
    const id = response.headers.get("X-Auth-Captcha-Id");
    if (!id || !response._data) throw new Error("Invalid challenge");
    if (src.value) URL.revokeObjectURL(src.value);
    src.value = URL.createObjectURL(response._data);
    emit("challenge", id);
  } catch {
    error.value = t(
      "验证码加载失败，点击重试。",
      "CAPTCHA unavailable. Click to retry.",
    );
  } finally {
    busy.value = false;
  }
}
onMounted(refresh);
watch(() => props.purpose, refresh);
onBeforeUnmount(() => {
  if (src.value) URL.revokeObjectURL(src.value);
});
defineExpose({ refresh });
</script>
<template>
  <button
    class="captcha"
    type="button"
    :disabled="busy"
    :aria-label="t('刷新图形验证码', 'Refresh CAPTCHA')"
    @click="refresh"
  >
    <img v-if="src" :src="src" :alt="t('图形验证码', 'CAPTCHA')" /><span
      v-else
      >{{ busy ? t("加载中…", "Loading…") : error }}</span
    >
  </button>
</template>
