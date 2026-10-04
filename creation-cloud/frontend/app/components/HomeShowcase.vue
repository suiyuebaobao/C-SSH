<script setup lang="ts">
import type { Section } from "~/types/site";
const props = defineProps<{ section: Section; en: boolean; label: string }>();
const selected = ref(0);
const panels = computed(() => [
  { key: 'monitor', item: props.section.items[2]!, image: 'product-monitor.png', label: props.en ? 'Monitoring' : '实时监控' },
  { key: 'terminal', item: props.section.items[1]!, image: 'product-terminal-session.png', label: props.en ? 'Terminal' : '持久终端' },
  { key: 'files', item: props.section.items[3]!, image: 'product-files.png', label: props.en ? 'Files' : '文件管理' },
  { key: 'ai', item: props.section.items[5]!, image: 'product-ai.png', label: props.en ? 'AI assistant' : 'AI 助手' },
]);
const active = computed(() => panels.value[selected.value]!);
function keyboard(event: KeyboardEvent) {
  const count = panels.value.length;
  if (event.key === 'ArrowRight') selected.value = (selected.value + 1) % count;
  else if (event.key === 'ArrowLeft') selected.value = (selected.value + count - 1) % count;
  else if (event.key === 'Home') selected.value = 0;
  else if (event.key === 'End') selected.value = count - 1;
  else return;
  event.preventDefault();
  (event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('[role=tab]')[selected.value]?.focus();
}
</script>
<template>
  <section id="product-showcase" class="home-showcase" :aria-label="label">
    <div class="showcase-toolbar">
      <span class="showcase-wordmark" aria-hidden="true">C-SSH <span>/</span> WORKSPACE</span>
      <div class="showcase-tabs" role="tablist" :aria-label="label" @keydown="keyboard">
        <button v-for="(panel, index) in panels" :id="'showcase-tab-' + panel.key" :key="panel.key" type="button" role="tab"
          :aria-selected="selected === index" :aria-controls="'showcase-panel-' + panel.key" :tabindex="selected === index ? 0 : -1" @click="selected = index">
          {{ panel.label }}
        </button>
      </div>
    </div>
    <div v-for="(panel, index) in panels" v-show="selected === index" :id="'showcase-panel-' + panel.key" :key="panel.key"
      class="showcase-panel" role="tabpanel" :aria-labelledby="'showcase-tab-' + panel.key" tabindex="0">
      <a :href="'/static/img/' + panel.image" target="_blank" rel="noopener noreferrer" :aria-label="(en ? 'View full image: ' : '查看完整图片：') + panel.label">
        <img :src="'/static/img/' + panel.image" :alt="panel.item.title + (en ? ' · demo data' : ' · 演示数据')" width="1440" height="960" :loading="index === 0 ? 'eager' : 'lazy'" :fetchpriority="index === 0 ? 'high' : 'auto'" />
      </a>
    </div>
    <div class="showcase-description" aria-live="polite"><strong>{{ active.item.title }}</strong><p>{{ active.item.body }}</p></div>
    <p class="showcase-caption">{{ en ? 'Product UI examples from the public gallery · demonstration data, not a live connection. Click an image for a closer look.' : '公开产品图集 · 界面使用演示数据，非实时连接。点击图片查看大图。' }}</p>
  </section>
</template>
