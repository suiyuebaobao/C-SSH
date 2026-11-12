<script setup lang="ts">
type Policy = {
  maximum: number | null;
  global_maximum: number;
  effective_maximum: number;
  active_devices: number;
  revision: number;
};
const props = defineProps<{ accountId: string }>();
const { t, en } = useLocale();
const api = useCloudApi();
const policy = ref<Policy | null>(null);
const inherit = ref(true);
const maximum = ref(16);
const busy = ref(false);
const error = ref("");
const notice = ref("");
const endpoint = computed(() => `/api/v1/admin/users/${props.accountId}/device-policy`);
const dirty = computed(() => policy.value !== null && (inherit.value ? null : maximum.value) !== policy.value.maximum);
let generation = 0;
function adopt(value: Policy) {
  policy.value = value;
  inherit.value = value.maximum === null;
  maximum.value = value.maximum ?? value.global_maximum;
}
async function load() {
  const ticket = ++generation;
  busy.value = true; error.value = ""; notice.value = "";
  try {
    const value = await api<Policy>(endpoint.value);
    if (ticket === generation) adopt(value);
  } catch (cause) {
    if (ticket === generation) error.value = cloudError(cause, en.value);
  } finally { if (ticket === generation) busy.value = false; }
}
async function save() {
  if (!policy.value || busy.value) return;
  if (!inherit.value && (!Number.isInteger(maximum.value) || maximum.value < 1 || maximum.value > 1000)) {
    error.value = t("设备上限必须是1到1000的整数。", "Enter a whole number from 1 to 1000."); return;
  }
  const ticket = generation;
  busy.value = true; error.value = ""; notice.value = "";
  try {
    const result = await api<Policy>(endpoint.value, "PUT", {
      maximum: inherit.value ? null : maximum.value,
      expected_revision: policy.value.revision,
    });
    if (ticket !== generation) return;
    adopt(result); notice.value = t("设备上限已保存。", "Device limit saved.");
  } catch (cause) { if (ticket === generation) error.value = cloudError(cause, en.value); }
  finally { if (ticket === generation) busy.value = false; }
}
watch(() => props.accountId, () => { policy.value = null; void load(); }, { immediate: true });
onBeforeUnmount(() => { generation++; });
</script>
<template>
  <section class="card">
    <h2>{{ t("客户端设备上限", "Client device limit") }}</h2>
    <p v-if="policy">{{ t("已授权设备", "Authorized devices") }}：{{ policy.active_devices }} / {{ policy.effective_maximum }}</p>
    <p>{{ t("同一设备的不同版本只占一个名额。调低上限只限制新增设备，不会立即撤销现有登录。", "Versions on the same device share one slot. Lowering the limit blocks new devices without revoking existing sign-ins.") }}</p>
    <form v-if="policy" @submit.prevent="save">
      <label><input v-model="inherit" type="checkbox" :disabled="busy" />{{ t("使用全站默认", "Use global default") }}（{{ policy.global_maximum }}）</label>
      <label v-if="!inherit">{{ t("允许的设备数量", "Allowed devices") }}<input v-model.number="maximum" type="number" min="1" max="1000" step="1" required :disabled="busy" /></label>
      <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
      <p v-if="error" role="alert" class="form-error">{{ error }}</p>
      <div class="form-actions"><button class="button primary" :disabled="busy || !dirty">{{ t("保存设备上限", "Save device limit") }}</button><button class="button" type="button" :disabled="busy" @click="load">{{ t("刷新", "Refresh") }}</button></div>
    </form>
    <template v-else><p v-if="error" role="alert" class="form-error">{{ error }}</p><button class="button" :disabled="busy" @click="load">{{ t("读取设备上限", "Load device limit") }}</button></template>
  </section>
</template>
