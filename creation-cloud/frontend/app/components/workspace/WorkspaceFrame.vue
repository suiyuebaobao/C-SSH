<script setup lang="ts">
const props = defineProps<{ admin?: boolean }>();
const session = useSessionStore();
const { t, en, publicPath } = useLocale();
const route = useRoute();
const api = useCloudApi();
const error = ref("");
const ready = ref(false);
const busy = ref(false);
const consoleLinks = [
  ["/console", "概览", "Overview"],
  ["/console/profile", "账号与安全", "Account & security"],
  ["/console/devices", "设备与会话", "Devices & sessions"],
  ["/console/hosts", "云端密文", "Encrypted data"],
  ["/console/models", "模型目录", "Models"],
  ["/console/downloads", "下载记录", "Downloads"],
  ["/feedback", "反馈", "Feedback"],
];
const adminLinks = [
  ["/admin", "总览", "Overview"],
  ["/admin/users", "用户", "Users"],
  ["/admin/devices", "设备与会话", "Devices & sessions"],
  ["/admin/releases", "版本管理", "Releases"],
  ["/admin/assets", "下载来源", "Assets"],
  ["/admin/announcements", "公告与通知", "Announcements"],
  ["/admin/models", "模型", "Models"],
  ["/admin/site", "网站内容", "Site content"],
  ["/admin/tutorials", "教学视频", "Tutorial videos"],
  ["/admin/seo", "搜索优化", "SEO"],
  ["/admin/feedback", "反馈", "Feedback"],
  ["/admin/audit", "审计", "Audit"],
  ["/admin/settings", "设置", "Settings"],
];
const links = computed(() => (props.admin ? adminLinks : consoleLinks));
useHead({ meta: [{ name: "robots", content: "noindex, nofollow" }] });
async function initialize() {
  try {
    if (!session.current) await session.refresh();
    if (props.admin && session.current?.role !== "admin") {
      error.value = t("没有管理员权限。", "Administrator access is required.");
      return;
    }
    ready.value = true;
  } catch (cause) {
    if ((cause as { status?: number }).status === 401)
      await navigateTo(
        `${publicPath("/login")}?next=${encodeURIComponent(route.path)}`,
      );
    else error.value = cloudError(cause, en.value);
  }
}
onMounted(initialize);
watch(
  () => session.current,
  async (value) => {
    if (!value && ready.value) {
      ready.value = false;
      await navigateTo(
        `${publicPath("/login")}?next=${encodeURIComponent(route.path)}`,
      );
    }
  },
);
async function logout() {
  if (busy.value) return;
  busy.value = true;
  try {
    await api("/api/v1/auth/logout", "POST");
    session.clear();
    await navigateTo(publicPath("/login"));
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
function link(path: string) {
  return en.value ? `${path}?lang=en` : path;
}
</script>
<template>
  <div class="workspace">
    <aside class="workspace-sidebar">
      <SiteLink :href="publicPath('/')" class="brand">C-SSH</SiteLink>
      <p class="eyebrow">{{ admin ? "ADMINISTRATION" : "CREATION CLOUD" }}</p>
      <nav>
        <NuxtLink
          v-for="item in links"
          :key="item[0]"
          :to="link(item[0]!)"
          :class="{ active: route.path === item[0] }"
          >{{ en ? item[2] : item[1] }}</NuxtLink
        ><NuxtLink
          v-if="!admin && session.current?.role === 'admin'"
          :to="link('/admin')"
          >{{ t("管理后台", "Administration") }} ↗</NuxtLink
        ><NuxtLink v-if="admin" :to="link('/console')"
          >{{ t("用户中心", "Console") }} ↗</NuxtLink
        >
      </nav>
      <button
        class="button"
        :disabled="busy || !session.current"
        @click="logout"
      >
        {{ t("退出登录", "Sign out") }}
      </button>
    </aside>
    <main id="main-content" class="workspace-main">
      <header class="workspace-header">
        <span>{{
          session.current?.email || session.current?.admin_login_name || "C-SSH"
        }}</span
        ><NuxtLink :to="`${route.path}${en ? '' : '?lang=en'}`">{{
          en ? "中文" : "English"
        }}</NuxtLink>
      </header>
      <p v-if="error" role="alert" class="form-error">{{ error }}</p>
      <slot v-if="ready" />
      <p v-else-if="!error">{{ t("正在验证会话…", "Checking session…") }}</p>
    </main>
  </div>
</template>
