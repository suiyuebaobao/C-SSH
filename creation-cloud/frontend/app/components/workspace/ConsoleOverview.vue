<script setup lang="ts">
import type { CloudPage, Profile } from "~/types/account";
const { t, en } = useLocale();
const api = useCloudApi();
const session = useSessionStore();
const profile = ref<Profile | null>(null);
const stats = ref<{ label: string; total: number; href: string }[]>([]);
const error = ref("");
onMounted(async () => {
  try {
    const [identity, devices, hosts, models, releases] = await Promise.all([
      api<Profile>(`/api/v1/users/${session.current!.account_id}`),
      api<CloudPage<unknown>>("/api/v1/devices?page=1&size=1"),
      api<CloudPage<unknown>>("/api/v1/hosts?page=1&size=1"),
      api<CloudPage<unknown>>("/api/v1/models?page=1&size=1"),
      api<unknown[]>("/api/v1/downloads/releases"),
    ]);
    profile.value = identity;
    stats.value = [
      {
        label: t("设备", "Devices"),
        total: devices.total,
        href: "/console/devices",
      },
      {
        label: t("加密主机记录", "Encrypted host records"),
        total: hosts.total,
        href: "/console/hosts",
      },
      {
        label: t("模型", "Models"),
        total: models.total,
        href: "/console/models",
      },
      {
        label: t("已发布版本", "Published releases"),
        total: releases.length,
        href: "/downloads",
      },
    ];
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
</script>
<template>
  <h1>{{ t("用户中心", "Your workspace") }}</h1>
  <p v-if="profile">{{ profile.display_name }}</p>
  <p v-if="error" class="form-error" role="alert">{{ error }}</p>
  <div class="stats-grid">
    <SiteLink
      v-for="stat in stats"
      :key="stat.href"
      :href="en ? `${stat.href}?lang=en` : stat.href"
      class="card"
      ><span>{{ stat.label }}</span
      ><strong>{{ stat.total }}</strong></SiteLink
    >
  </div>
  <aside class="notice">
    <h2>{{ t("密文由客户端管理", "Encryption stays on your device") }}</h2>
    <p>
      {{
        t(
          "网站不会读取主机地址、SSH 凭据、AI Key 或客户端对话。数据上传和下载需在客户端主动操作。",
          "The website does not read host addresses, SSH credentials, AI keys or client conversations. Upload and download encrypted data explicitly from the client.",
        )
      }}
    </p>
  </aside>
</template>
