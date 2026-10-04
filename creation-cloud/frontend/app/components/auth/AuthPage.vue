<script setup lang="ts">
import type { Session } from "~/types/account";
const props = defineProps<{ mode: string }>();
const { en, t, publicPath } = useLocale();
const route = useRoute();
const session = useSessionStore();
const api = useCloudApi();
const next = computed(() =>
  typeof route.query.next === "string" &&
  (route.query.next === "/feedback" ||
    /^\/(admin|console)(\/[-a-z0-9/]*)?$/.test(route.query.next))
    ? route.query.next
    : "/console",
);
const admin = computed(() => next.value.startsWith("/admin"));
const identifier = ref("");
const email = ref(session.email);
const password = ref("");
const displayName = ref("");
const code = ref("");
const captchaId = ref("");
const captchaCode = ref("");
const captcha = useTemplateRef("captcha");
const policy = ref<{
  user_captcha_enabled: boolean;
  admin_captcha_enabled: boolean;
} | null>(null);
const busy = ref(false);
const error = ref("");
const notice = ref("");
const policyModes = ["login", "register", "forgot-password"];
const needsPolicy = computed(() => policyModes.includes(props.mode));
const captchaRequired = computed(
  () =>
    needsPolicy.value &&
    (admin.value && props.mode === "login"
      ? policy.value?.admin_captcha_enabled
      : policy.value?.user_captcha_enabled),
);
const purpose = computed(() =>
  props.mode === "register"
    ? "register"
    : props.mode === "forgot-password"
      ? "password_reset"
      : admin.value
        ? "admin_login"
        : "login",
);
const titles: Record<string, [string, string]> = {
  login: ["登录", "Sign in"],
  register: ["注册账号", "Create an account"],
  "verify-email": ["验证邮箱", "Verify email"],
  "verify-login": ["验证登录", "Verify sign-in"],
  "forgot-password": ["找回密码", "Forgot password"],
  "reset-password": ["重置密码", "Reset password"],
};
const title = computed(() => titles[props.mode]?.[en.value ? 1 : 0] ?? "C-SSH");
useHead(() => ({
  title: `${title.value} · C-SSH`,
  meta: [{ name: "robots", content: "noindex, nofollow" }],
}));
async function loadPolicy() {
  error.value = "";
  try {
    policy.value = await api("/api/v1/auth/policy");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
onMounted(() => {
  if (needsPolicy.value) void loadPolicy();
});
function authPath(path: string) {
  return `${publicPath(path)}${admin.value ? "?next=/admin" : ""}`;
}
async function finish(
  result: Session | { challenge_id?: string; status: string },
) {
  if ("account_id" in result) {
    session.current = result;
    session.email = "";
    session.challenge = "";
    const destination =
      next.value === "/feedback"
        ? publicPath("/feedback")
        : `${next.value}${en.value ? "?lang=en" : ""}`;
    await navigateTo(destination);
  } else if (result.challenge_id) {
    session.challenge = result.challenge_id;
    await navigateTo(authPath("/verify-login"));
  } else {
    session.email = email.value;
    await navigateTo(authPath("/verify-email"));
  }
}
async function submit() {
  if (busy.value || (needsPolicy.value && !policy.value)) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    const challenge = captchaRequired.value
      ? { captcha_id: captchaId.value, captcha_code: captchaCode.value }
      : {};
    if (props.mode === "login") {
      await finish(
        await api("/api/v1/auth/login", "POST", {
          identifier: identifier.value,
          password: password.value,
          ...challenge,
        }),
      );
    } else if (props.mode === "register") {
      await finish(
        await api("/api/v1/auth/register", "POST", {
          email: email.value,
          password: password.value,
          display_name: displayName.value,
          locale: en.value ? "en" : "zh-CN",
          ...challenge,
        }),
      );
    } else if (props.mode === "verify-email") {
      await finish(
        await api("/api/v1/auth/verify-email", "POST", {
          email: email.value,
          code: code.value,
        }),
      );
    } else if (props.mode === "verify-login") {
      if (!session.challenge) {
        error.value = t(
          "验证上下文已失效，请重新登录。",
          "Start sign-in again to obtain a verification challenge.",
        );
        return;
      }
      await finish(
        await api("/api/v1/auth/verify-login", "POST", {
          challenge_id: session.challenge,
          code: code.value,
        }),
      );
    } else if (props.mode === "forgot-password") {
      await api("/api/v1/auth/password-reset/request", "POST", {
        email: email.value,
        ...challenge,
      });
      session.email = email.value;
      await navigateTo(publicPath("/reset-password"));
    } else if (props.mode === "reset-password") {
      await api("/api/v1/auth/password-reset/confirm", "POST", {
        email: email.value,
        code: code.value,
        new_password: password.value,
      });
      session.clear();
      notice.value = t(
        "密码已重置，请重新登录。",
        "Password reset. Sign in again.",
      );
    }
  } catch (cause) {
    error.value = cloudError(cause, en.value);
    if (captchaRequired.value) await captcha.value?.refresh();
  } finally {
    password.value = "";
    code.value = "";
    captchaCode.value = "";
    busy.value = false;
  }
}
async function resend() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    if (props.mode === "verify-login") {
      const result = await api<{ challenge_id: string }>(
        "/api/v1/auth/resend-login-verification",
        "POST",
        { challenge_id: session.challenge },
      );
      session.challenge = result.challenge_id;
    } else
      await api("/api/v1/auth/resend-verification", "POST", {
        email: email.value,
      });
    notice.value = t(
      "请求已受理，请检查邮箱并遵守发送间隔。",
      "Request accepted. Check your email and respect the resend interval.",
    );
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <main class="container auth-page">
    <SiteLink :href="publicPath('/')" class="brand">C-SSH</SiteLink>
    <div class="card auth-card">
      <p class="eyebrow">
        {{ admin ? t("管理后台", "Administration") : "CREATION CLOUD" }}
      </p>
      <h1>{{ title }}</h1>
      <p v-if="mode === 'forgot-password' || mode === 'reset-password'">
        {{
          t(
            "只重置网站账号密码，不恢复客户端数据保护密码。",
            "Resets your website password, not your client data-protection password.",
          )
        }}
      </p>
      <p
        v-if="mode === 'verify-login' && !session.challenge"
        class="form-error"
      >
        {{
          t(
            "页面刷新后请重新登录，验证信息不会保存在浏览器。",
            "After a page reload, start sign-in again. Verification data is not persisted.",
          )
        }}
      </p>
      <form @submit.prevent="submit">
        <label v-if="mode === 'login'"
          >{{
            admin
              ? t("管理员登录名", "Administrator login name")
              : t("邮箱", "Email")
          }}<input
            v-model="identifier"
            autocomplete="username"
            required
            maxlength="254"
        /></label>
        <label
          v-if="
            [
              'register',
              'verify-email',
              'forgot-password',
              'reset-password',
            ].includes(mode)
          "
          >{{ t("邮箱", "Email")
          }}<input
            v-model="email"
            type="email"
            autocomplete="email"
            required
            maxlength="254"
        /></label>
        <label v-if="mode === 'register'"
          >{{ t("显示名称", "Display name")
          }}<input
            v-model="displayName"
            autocomplete="nickname"
            required
            maxlength="80"
        /></label>
        <label v-if="['login', 'register', 'reset-password'].includes(mode)"
          >{{ t("密码", "Password")
          }}<input
            v-model="password"
            type="password"
            :autocomplete="
              mode === 'login' ? 'current-password' : 'new-password'
            "
            required
            maxlength="1024"
        /></label>
        <label
          v-if="
            ['verify-email', 'verify-login', 'reset-password'].includes(mode)
          "
          >{{ t("邮箱验证码", "Email verification code")
          }}<input
            v-model="code"
            inputmode="numeric"
            autocomplete="one-time-code"
            pattern="[0-9]{6}"
            maxlength="6"
            required
        /></label>
        <template v-if="captchaRequired"
          ><AuthCaptchaImage
            ref="captcha"
            :purpose="purpose"
            @challenge="
              (id) => {
                captchaId = id;
                captchaCode = '';
              }
            " /><label
            >{{ t("图形验证码", "CAPTCHA code")
            }}<input
              v-model="captchaCode"
              autocomplete="off"
              required
              maxlength="16" /></label
        ></template>
        <p v-if="error" role="alert" class="form-error">{{ error }}</p>
        <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
        <button
          class="button primary"
          :disabled="
            busy ||
            (needsPolicy && !policy) ||
            (mode === 'verify-login' && !session.challenge) ||
            (!!captchaRequired && !captchaId)
          "
        >
          {{ busy ? t("处理中…", "Working…") : title }}
        </button>
        <button
          v-if="needsPolicy && !policy"
          class="button"
          type="button"
          @click="loadPolicy"
        >
          {{ t("重新加载", "Reload") }}
        </button>
        <button
          v-if="
            mode === 'verify-email' ||
            (mode === 'verify-login' && session.challenge)
          "
          class="button"
          type="button"
          :disabled="busy"
          @click="resend"
        >
          {{ t("重新发送验证码", "Resend code") }}
        </button>
      </form>
      <nav class="auth-links">
        <NuxtLink :to="authPath('/login')">{{ t("登录", "Sign in") }}</NuxtLink
        ><NuxtLink :to="publicPath('/register')">{{
          t("注册", "Register")
        }}</NuxtLink
        ><NuxtLink :to="publicPath('/forgot-password')">{{
          t("忘记密码", "Forgot password")
        }}</NuxtLink
        ><NuxtLink v-if="!admin" :to="`${publicPath('/login')}?next=/admin`">{{
          t("管理员", "Administrator")
        }}</NuxtLink>
      </nav>
    </div>
  </main>
</template>
