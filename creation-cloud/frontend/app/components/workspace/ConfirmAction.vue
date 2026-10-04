<script setup lang="ts">
const props = defineProps<{
  label: string;
  message: string;
  endpoint: string;
  method?: "POST" | "DELETE" | "PATCH";
  body?: Record<string, unknown>;
  disabled?: boolean;
}>();
const emit = defineEmits<{ done: [] }>();
const { en, t } = useLocale();
const api = useCloudApi();
const opened = ref(false);
const busy = ref(false);
const error = ref("");
async function perform() {
  if (busy.value || props.disabled) return;
  busy.value = true;
  error.value = "";
  try {
    await api(props.endpoint, props.method ?? "DELETE", props.body);
    opened.value = false;
    emit("done");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <button type="button" class="button danger" :disabled="disabled" @click="opened = true">
    {{ label }}</button
  ><Teleport to="body"
    ><div v-if="opened" class="modal-overlay">
      <section
        class="modal card"
        role="alertdialog"
        aria-modal="true"
        :aria-label="label"
      >
        <h2>{{ label }}</h2>
        <p>{{ message }}</p>
        <p v-if="error" role="alert" class="form-error">{{ error }}</p>
        <div class="actions">
          <button
            type="button"
            class="button"
            :disabled="busy"
            @click="opened = false"
          >
            {{ t("取消", "Cancel") }}</button
          ><button
            type="button"
            class="button danger"
            :disabled="busy"
            @click="perform"
          >
            {{ busy ? t("处理中…", "Working…") : t("确认", "Confirm") }}
          </button>
        </div>
      </section>
    </div></Teleport
  >
</template>
