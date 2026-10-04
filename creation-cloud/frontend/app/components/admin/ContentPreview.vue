<script setup lang="ts">
import type { Home, Page, Shell } from "~/types/site";
const props = defineProps<{
  payload: { document_key: string; content: Record<string, unknown> };
}>();
const home = computed(() => props.payload.content.content as Home);
const page = computed<Page>(() => ({
  id: "Home",
  heading: String(props.payload.content.heading),
  eyebrow: String(props.payload.content.eyebrow),
  lead: String(props.payload.content.lead),
  meta_title: "",
  meta_description: "",
  actions: props.payload.content.actions as Page["actions"],
  sections: [],
  faqs: [],
  home_page: home.value,
  documentation_page: null,
}));
</script>
<template>
  <div class="content-preview">
    <template v-if="payload.document_key === 'home'"
      ><PageHeading :page="page" /><ContentSections :sections="home.sections" />
      <h2>{{ home.faq_heading }}</h2>
      <FaqList :items="home.faqs" />
      <section class="closing">
        <h2>{{ home.final_heading }}</h2>
        <p>{{ home.final_lead }}</p>
      </section></template
    ><template v-else
      ><h2>C-SSH</h2>
      <p>{{ payload.content.brand_note }}</p>
      <nav class="actions">
        <SiteLink
          v-for="item in payload.content.navigation as Shell['navigation']"
          :key="item.href"
          :href="item.href"
          >{{ item.label }}</SiteLink
        >
      </nav>
      <p>{{ payload.content.footer_summary }}</p>
      <p>{{ payload.content.footer_signature }}</p></template
    >
  </div>
</template>
