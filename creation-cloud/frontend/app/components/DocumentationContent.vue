<script setup lang="ts">
import type { SiteDocument } from "~/types/site";
const props = defineProps<{ document: SiteDocument }>();
const docs = computed(() => props.document.view.page.documentation_page!);
const query = ref("");
const filtered = computed(() =>
  docs.value.groups
    .map((group) => ({
      ...group,
      links: group.links.filter((link) =>
        `${link.code} ${link.title}`
          .toLocaleLowerCase()
          .includes(query.value.trim().toLocaleLowerCase()),
      ),
    }))
    .filter((group) => group.links.length),
);
</script>
<template>
  <main id="main-content" class="container docs-layout">
    <aside class="docs-sidebar">
      <label for="docs-search">{{ docs.search_label }}</label
      ><input
        id="docs-search"
        v-model="query"
        type="search"
        :placeholder="docs.search_placeholder"
      />
      <p>{{ docs.search_help }}</p>
      <SiteLink :href="document.view.shell.locale === 'en' ? '/en/tutorials' : '/tutorials'" class="button">{{ document.view.shell.locale === 'en' ? 'Video tutorials' : '视频教学' }} →</SiteLink>
      <nav :aria-label="docs.index_label">
        <section v-for="group in filtered" :key="group.title">
          <h2>{{ group.title }}</h2>
          <a
            v-for="link in group.links"
            :key="link.anchor"
            :href="`#${link.anchor}`"
            ><span>{{ link.code }}</span
            >{{ link.title }}</a
          >
        </section>
        <p v-if="!filtered.length" role="status">{{ docs.search_empty }}</p>
      </nav>
    </aside>
    <article class="docs-article">
      <PageHeading :page="document.view.page" />
      <aside class="notice">
        <span class="eyebrow">{{ docs.status.label }}</span>
        <h2>{{ docs.status.title }}</h2>
        <p>{{ docs.status.body }}</p>
      </aside>
      <section
        v-for="section in docs.sections"
        :id="section.anchor"
        :key="section.anchor"
        class="content-section"
      >
        <p class="eyebrow">{{ section.code }}</p>
        <h2>{{ section.title }}</h2>
        <p>{{ section.lead }}</p>
        <article
          v-for="item in section.items"
          :key="item.title"
          :class="['doc-step', { notice: item.caution }]"
        >
          <span class="eyebrow">{{ item.badge }}</span>
          <h3>{{ item.title }}</h3>
          <p>{{ item.body }}</p>
          <pre v-if="item.has_command"><code>{{ item.command }}</code></pre>
        </article>
      </section>
      <h2>{{ docs.tutorials.index_label }}</h2>
      <section
        v-for="tutorial in docs.tutorials.items"
        :id="tutorial.anchor"
        :key="tutorial.anchor"
        class="content-section"
      >
        <p class="eyebrow">{{ tutorial.sequence }}</p>
        <h2>{{ tutorial.title }}</h2>
        <p>{{ tutorial.summary }}</p>
        <h3>{{ docs.tutorials.prerequisites_label }}</h3>
        <ul>
          <li v-for="item in tutorial.prerequisites" :key="item">{{ item }}</li>
        </ul>
        <h3>{{ docs.tutorials.steps_label }}</h3>
        <ol class="tutorial-steps">
          <li v-for="step in tutorial.steps" :key="step.label">
            <h4>{{ step.title }}</h4>
            <p>{{ step.instruction }}</p>
          </li>
        </ol>
        <div class="notice">
          <h3>{{ docs.tutorials.verification_label }}</h3>
          <p>{{ tutorial.verification }}</p>
          <h3>{{ docs.tutorials.agent_boundary_label }}</h3>
          <p>{{ tutorial.agent_boundary }}</p>
        </div>
      </section>
      <figure class="doc-screenshot">
        <h2>{{ docs.screenshot.title }}</h2>
        <p>{{ docs.screenshot.lead }}</p>
        <img
          :src="docs.screenshot.src"
          :alt="docs.screenshot.alt"
          :width="docs.screenshot.width"
          :height="docs.screenshot.height"
          loading="lazy"
        />
        <figcaption>{{ docs.screenshot.caption }}</figcaption>
      </figure>
      <section class="closing">
        <h2>{{ docs.final_title }}</h2>
        <p>{{ docs.final_body }}</p>
      </section>
    </article>
  </main>
</template>
