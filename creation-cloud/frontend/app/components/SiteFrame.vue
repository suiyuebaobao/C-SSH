<script setup lang="ts">
import type { Shell } from "~/types/site";
defineProps<{ shell: Shell }>();
const navigation = useNavigationStore();
</script>
<template>
  <div class="site-frame">
    <a class="skip" href="#main-content">{{ shell.skip_label }}</a>
    <header class="site-header">
      <div class="container navbar">
        <SiteLink :href="shell.home_href" class="brand"
          ><span class="brand-symbol" aria-hidden="true">C<span>›</span></span
          >C-SSH</SiteLink
        >
        <button
          class="menu-button"
          :aria-expanded="navigation.menuOpen"
          aria-controls="site-navigation"
          @click="navigation.toggle()"
        >
          {{ shell.menu_label }}
        </button>
        <nav
          id="site-navigation"
          :class="{ open: navigation.menuOpen }"
          :aria-label="shell.menu_label"
        >
          <SiteLink
            v-for="item in shell.navigation"
            :key="item.href"
            :href="item.href"
            @click="navigation.close()"
            >{{ item.label }}</SiteLink
          >
          <SiteLink :href="shell.locale === 'en' ? '/en/tutorials' : '/tutorials'">{{ shell.locale === 'en' ? 'Videos' : '视频教学' }}</SiteLink>
          <SiteLink
            :href="shell.github_repository.href"
            target="_blank"
            rel="noopener noreferrer"
            referrerpolicy="no-referrer"
            class="github-repository"
            >{{ shell.github_repository.label }} ↗</SiteLink
          >
          <SiteLink :href="shell.language_href" class="language">{{
            shell.language_label
          }}</SiteLink>
          <SiteLink :href="shell.console_link.href" class="button subtle"
            >{{ shell.console_link.label }} ↗</SiteLink
          >
        </nav>
      </div>
    </header>
    <slot />
    <footer class="site-footer container">
      <div>
        <SiteLink :href="shell.home_href" class="brand">C-SSH</SiteLink>
        <p>{{ shell.footer_summary }}</p>
        <small>{{ shell.footer_note }}</small>
      </div>
      <nav :aria-label="shell.menu_label">
        <SiteLink
          v-for="item in shell.footer_navigation"
          :key="item.href"
          :href="item.href"
          >{{ item.label }}</SiteLink
        ><SiteLink :href="shell.github_repository.href">GitHub ↗</SiteLink>
      </nav>
      <p class="footer-signature">{{ shell.footer_motto }}</p>
    </footer>
  </div>
</template>
