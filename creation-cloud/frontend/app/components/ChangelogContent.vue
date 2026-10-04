<script setup lang="ts">
import type { SiteDocument } from "~/types/site";
defineProps<{ document: SiteDocument }>();
</script>
<template>
  <main id="main-content" class="container narrow">
    <PageHeading :page="document.view.page" />
    <div v-if="document.catalog?.releases.length" class="timeline">
      <article
        v-for="release in document.catalog.releases"
        :key="release.version"
        class="release-entry"
      >
        <header>
          <span class="release-version">v{{ release.version }}</span
          ><time :datetime="release.published_at_iso">{{
            release.published_at_label
          }}</time>
        </header>
        <h2>{{ release.title }}</h2>
        <div class="release-notes">
          <template v-for="(block, index) in release.notes_blocks" :key="index"
            ><h3 v-if="block.kind === 'heading'">{{ block.content }}</h3>
            <ul v-else-if="block.kind === 'bullets'">
              <li v-for="(item, i) in block.content" :key="i">{{ item }}</li>
            </ul>
            <p v-else>{{ block.content }}</p></template
          >
        </div>
      </article>
    </div>
    <p v-else class="empty-state">
      {{
        document.view.shell.locale === "en"
          ? "No published releases available."
          : "暂时没有已发布版本。"
      }}
    </p>
  </main>
</template>
