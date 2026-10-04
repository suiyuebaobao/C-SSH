<script setup lang="ts">
import type { SiteDocument } from "~/types/site";
const props = defineProps<{ document: SiteDocument }>();
const en = computed(() => props.document.view.shell.locale === "en");
const latest = computed(() => props.document.catalog?.latest);
const selection = ref("all");
const platforms = computed(() => [
  ...(latest.value?.platforms ?? []),
  ...(latest.value?.manual_platforms ?? []),
]);
const visible = computed(() =>
  platforms.value.filter(
    (platform) =>
      selection.value === "all" || platform.name === selection.value,
  ),
);
</script>
<template>
  <main id="main-content" class="container">
    <PageHeading :page="document.view.page" />
    <template v-if="latest">
      <aside v-if="latest.manual_notice" class="notice">
        <strong>{{ latest.manual_notice.title }}</strong>
        <p>{{ latest.manual_notice.body }}</p>
        <SiteLink :href="latest.manual_notice.link.href"
          >{{ latest.manual_notice.link.label }} ↗</SiteLink
        >
      </aside>
      <div class="filter-bar">
        <label for="platform-select">{{ en ? "Platform" : "平台" }}</label
        ><select id="platform-select" v-model="selection">
          <option value="all">{{ en ? "All platforms" : "全部平台" }}</option>
          <option
            v-for="platform in platforms"
            :key="platform.name"
            :value="platform.name"
          >
            {{ platform.name }}
          </option></select
        ><span>v{{ latest.version }} · {{ latest.channel }}</span>
      </div>
      <div class="download-grid">
        <section
          v-for="platform in visible"
          :key="platform.name"
          class="card download-card"
        >
          <h2>{{ platform.name }}</h2>
          <p v-if="'note' in platform">{{ platform.note }}</p>
          <div class="download-options">
            <template
              v-for="asset in platform.assets"
              :key="`${asset.architecture}:${asset.package_kind}`"
              ><SiteLink
                v-if="asset.download_href"
                class="download-option"
                :href="asset.download_href"
                ><span
                  ><strong>{{ asset.package_kind }}</strong
                  ><small>{{ asset.architecture }}</small></span
                ><span aria-hidden="true">↓</span></SiteLink
              >
              <p v-else>
                {{ asset.package_kind }} · {{ en ? "Unavailable" : "暂不可用" }}
              </p></template
            >
          </div>
        </section>
      </div>
      <p class="empty-note">iOS · {{ en ? "Not published" : "尚未发布" }}</p>
    </template>
    <p v-else class="empty-state">
      {{ en ? "No download is available yet." : "暂时没有可用下载。" }}
    </p>
  </main>
</template>
