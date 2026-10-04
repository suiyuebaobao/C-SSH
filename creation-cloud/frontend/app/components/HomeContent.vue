<script setup lang="ts">
import type { SiteDocument } from "~/types/site";
const props = defineProps<{ document: SiteDocument }>();
const home = computed(() => props.document.view.page.home_page!);
const en = computed(() => props.document.view.shell.locale === "en");
const capabilities = computed(() => home.value.sections.find(s => s.anchor === "capabilities"));
const cloud = computed(() => home.value.sections.find(s => s.anchor === "cloud"));
const security = computed(() => home.value.sections.find(s => s.anchor === "security"));
const highlights = computed(() => [0, 4, 7].map(index => capabilities.value?.items[index]).filter(Boolean));
const cloudItems = computed(() => [1, 2].map(index => cloud.value?.items[index]).filter(Boolean));
const headingLines = computed(() => {
  const heading = props.document.view.page.heading;
  const separator = en.value ? ". " : "，";
  const index = heading.indexOf(separator);
  return index < 0 ? [heading] : [heading.slice(0, index + separator.trim().length), heading.slice(index + separator.length)];
});
const downloads = computed(() => en.value ? "/en/downloads" : "/downloads");
const docs = computed(() => en.value ? "/en/docs/getting-started" : "/docs/getting-started");
</script>
<template>
  <main id="main-content" class="product-home">
    <section class="home-hero container">
      <p class="eyebrow">{{ document.view.page.eyebrow }}</p>
      <h1><span v-for="(line, index) in headingLines" :key="index" class="home-title-line">{{ line }}</span></h1>
      <p class="hero-description">{{ document.view.page.lead }}</p>
      <div class="hero-actions">
        <SiteLink v-for="(action, index) in document.view.page.actions" :key="action.href"
          :href="action.href" :class="['button', { primary: index === 0, 'hero-text-link': index === 2 }]">
          {{ action.label }} <span aria-hidden="true">{{ index === 0 ? '↓' : '↗' }}</span>
        </SiteLink>
      </div>
      <SiteLink :href="downloads" class="home-release-note"><span aria-hidden="true"></span>{{ home.status_note }} <span aria-hidden="true">→</span></SiteLink>
    </section>

    <div class="container">
      <HomeShowcase v-if="capabilities" :section="capabilities" :en="en" :label="home.hero_blueprint_label" />
      <section class="home-video-entry">
        <span aria-hidden="true">▶</span>
        <div><p class="eyebrow">{{ en ? 'WATCH & LEARN' : '跟着实际操作，快速上手' }}</p>
          <h2>{{ en ? 'See your next workflow in action.' : '从连接主机，到完成日常工作。' }}</h2>
          <p>{{ en ? 'Learn host setup, daily operations, AI and cloud through recorded client workflows.' : '通过客户端实录，学习主机接入、日常运维、AI 与云端。' }}</p>
        </div>
        <SiteLink :href="en ? '/en/tutorials' : '/tutorials'" class="button primary">{{ en ? 'Watch tutorials' : '观看视频教学' }} →</SiteLink>
      </section>
      <section id="platforms" class="home-platforms" :aria-label="home.platform_label">
        <p>{{ home.platform_label }}</p>
        <div class="home-platform-options">
          <SiteLink v-for="platform in home.platforms" :key="platform.name" :href="downloads">
            <strong>{{ platform.name }}</strong><span>{{ platform.detail }}</span>
          </SiteLink>
        </div>
      </section>

      <section v-if="capabilities" id="capabilities" class="home-capabilities">
        <div class="home-section-intro">
          <p class="eyebrow">{{ en ? 'THE DAILY WORK' : '从连接，到完成工作' }}</p>
          <h2>{{ capabilities.title }}</h2><p>{{ capabilities.lead }}</p>
        </div>
        <div class="home-feature-lines">
          <article v-for="(item, index) in highlights" :key="index">
            <span class="home-line-number" aria-hidden="true">0{{ index + 1 }}</span>
            <div><h3>{{ item!.title }}</h3><p>{{ item!.body }}</p></div>
          </article>
        </div>
        <SiteLink :href="en ? 'https://github.com/suiyuebaobao/C-SSH/blob/main/FEATURES_EN.md' : 'https://github.com/suiyuebaobao/C-SSH/blob/main/FEATURES.md'" class="home-inline-link">
          {{ en ? 'Explore all features' : '查看完整功能' }} <span aria-hidden="true">↗</span>
        </SiteLink>
      </section>

      <section v-if="cloud" id="cloud" class="home-cloud">
        <div>
          <p class="eyebrow">{{ en ? 'YOUR WORKSPACE, YOUR CHOICE' : '自己的工作台，自己的选择' }}</p>
          <h2>{{ cloud.title }}</h2><p>{{ cloud.lead }}</p>
          <SiteLink :href="docs" class="home-inline-link">{{ en ? 'Read the getting-started guide' : '阅读上手指南' }} ↗</SiteLink>
        </div>
        <div class="home-cloud-points">
          <article v-for="(item, index) in cloudItems" :key="index"><span aria-hidden="true">{{ index === 0 ? '↗' : '⇄' }}</span><div><h3>{{ item!.title }}</h3><p>{{ item!.body }}</p></div></article>
          <p v-if="security" class="home-security-note">{{ security.lead }} <SiteLink :href="en ? '/en/security' : '/security'">{{ en ? 'Data protection' : '了解数据保护' }} ↗</SiteLink></p>
        </div>
      </section>

      <section class="home-faq">
        <div><p class="eyebrow">FAQ</p><h2>{{ home.faq_heading }}</h2><SiteLink :href="en ? '/en/faq' : '/faq'" class="home-inline-link">{{ en ? 'More questions' : '更多常见问题' }} ↗</SiteLink></div>
        <FaqList :items="home.faqs.slice(0, 3)" />
      </section>
      <section class="home-final">
        <p class="eyebrow">C-SSH</p><h2>{{ home.final_heading }}</h2><p>{{ home.final_lead }}</p>
        <SiteLink :href="downloads" class="button primary">{{ en ? 'Download C-SSH' : '下载 C-SSH' }} <span aria-hidden="true">↓</span></SiteLink>
      </section>
    </div>
    <HomeContact :widget="home.qr_widget" />
  </main>
</template>
<style src="../assets/css/homepage.css"></style>
