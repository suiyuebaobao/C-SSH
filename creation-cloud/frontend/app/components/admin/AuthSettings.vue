<script setup lang="ts">
import type { FormField } from "~/types/forms";
const { t, en } = useLocale();
const api = useCloudApi();
const form = ref<Record<string, unknown>>({});
const revision = ref<number | null>(null);
const busy = ref(false),
  error = ref(""),
  notice = ref("");
const fields: FormField[] = [
  {
    key: "email_verification_enabled",
    label: ["用户邮箱验证码", "User email verification"],
    kind: "checkbox",
  },
  {
    key: "user_captcha_enabled",
    label: ["用户图形验证码", "User CAPTCHA"],
    kind: "checkbox",
  },
  {
    key: "admin_email_verification_enabled",
    label: ["管理员邮箱验证码", "Admin email verification"],
    kind: "checkbox",
  },
  {
    key: "admin_captcha_enabled",
    label: ["管理员图形验证码", "Admin CAPTCHA"],
    kind: "checkbox",
  },
  {
    key: "email_cooldown_seconds",
    label: ["邮件获取间隔（秒）", "Email resend interval (seconds)"],
    kind: "number",
    min: 30,
    max: 3600,
    required: true,
  },
  {
    key: "login_failure_threshold",
    label: ["连续密码失败次数", "Password failure limit"],
    kind: "number",
    min: 3,
    max: 20,
    required: true,
  },
  {
    key: "login_lockout_minutes",
    label: ["登录锁定时长（分钟）", "Lockout duration (minutes)"],
    kind: "number",
    min: 1,
    max: 1440,
    required: true,
  },
];
async function load() {
  try {
    const data = await api<Record<string, unknown>>(
      "/api/v1/admin/auth-settings",
    );
    revision.value = Number(data.revision);
    form.value = Object.fromEntries(
      fields.map((field) => [field.key, data[field.key]]),
    );
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
onMounted(load);
async function save() {
  if (busy.value || revision.value === null) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    const data = await api<{ revision: number }>(
      "/api/v1/admin/auth-settings",
      "PUT",
      { ...form.value, expected_revision: revision.value },
    );
    revision.value = data.revision;
    notice.value = t("设置已保存。", "Settings saved.");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="card">
    <h2>{{ t("认证与安全", "Authentication & security") }}</h2>
    <p>
      {{
        t(
          "管理员邮箱码和图形码至少保留一项。关闭用户邮箱码会使未完成的验证挑战失效。",
          "Keep at least one admin verification method enabled. Disabling user email verification invalidates pending challenges.",
        )
      }}
    </p>
    <form v-if="revision !== null" @submit.prevent="save">
      <AdminFieldsEditor v-model="form" :fields="fields" />
      <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
      <button class="button primary" :disabled="busy">
        {{ t("保存", "Save") }}
      </button>
    </form>
    <p v-if="error" role="alert" class="form-error">{{ error }}</p>
    <button class="button" :disabled="busy" @click="load">
      {{ t("刷新设置", "Refresh settings") }}
    </button>
  </section>
</template>
