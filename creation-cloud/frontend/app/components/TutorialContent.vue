<script setup lang="ts">
import type { SiteDocument } from "~/types/site";
import type { TutorialVideo } from "~/types/tutorials";
const props = defineProps<{ document: SiteDocument }>();
const locale = props.document.view.shell.locale;
const en = locale === "en";
const config = useRuntimeConfig();
const baseURL = import.meta.server ? config.rustOrigin : undefined;
const { data, error } = await useAsyncData(`tutorials:${locale}`, () =>
  $fetch<TutorialVideo[]>("/api/v1/tutorials", {
    baseURL,
    query: { locale },
    credentials: "omit",
    retry: 0,
    timeout: 10000,
  }),
);
if (error.value || !data.value)
  throw createError({
    statusCode: 503,
    statusMessage: "Tutorials unavailable",
  });
const videos = ref(data.value);
const selected = ref(videos.value[0] ?? null);
const player = useTemplateRef<HTMLVideoElement>("player");
const watchArea = useTemplateRef<HTMLElement>("watchArea");
const playbackError = ref("");
const loading = ref(false);
const more = ref(videos.value.length === 100);
const loadError = ref("");
async function choose(video: TutorialVideo) {
  selected.value = video;
  playbackError.value = "";
  await nextTick();
  watchArea.value?.scrollIntoView({ behavior: "smooth", block: "start" });
  player.value?.focus({ preventScroll: true });
  try {
    await player.value?.play();
  } catch {
    playbackError.value = en
      ? "Press play in the player to begin."
      : "请点击播放器上的播放按钮。";
  }
}
async function loadMore() {
  if (loading.value) return;
  loading.value = true;
  loadError.value = "";
  try {
    const next = await $fetch<TutorialVideo[]>("/api/v1/tutorials", {
      query: { locale, offset: videos.value.length },
      credentials: "omit",
      retry: 0,
    });
    videos.value.push(...next);
    more.value = next.length === 100;
  } catch {
    loadError.value = en
      ? "More videos could not be loaded."
      : "暂时无法读取更多影片。";
  } finally {
    loading.value = false;
  }
}
</script>
<template>
  <main id="main-content" class="container tutorial-page">
    <PageHeading :page="document.view.page" />
    <nav
      class="tutorial-languages"
      :aria-label="en ? 'Video language' : '影片语言'"
    >
      <SiteLink href="/tutorials" :aria-current="!en ? 'page' : undefined"
        >中文</SiteLink
      >
      <SiteLink href="/en/tutorials" :aria-current="en ? 'page' : undefined"
        >English</SiteLink
      >
    </nav>
    <section
      v-if="selected"
      ref="watchArea"
      class="tutorial-watch"
      aria-labelledby="tutorial-title"
    >
      <video
        :key="selected.id"
        ref="player"
        :src="selected.video_url"
        :poster="selected.poster_url"
        controls
        playsinline
        preload="none"
        tabindex="0"
        :aria-label="selected.title"
        @error="
          playbackError = en
            ? 'This video could not be loaded. Refresh the page to check its availability.'
            : '影片暂时无法播放，请刷新页面确认是否仍公开。'
        "
      >
        {{
          en
            ? "Your browser does not support this video player."
            : "浏览器不支持此视频播放器。"
        }}
      </video>
      <div class="tutorial-caption">
        <p class="eyebrow">
          {{ en ? "C-SSH FEATURE TOUR" : "C-SSH 操作教学" }}
        </p>
        <h2 id="tutorial-title">{{ selected.title }}</h2>
        <p v-if="selected.version_note" class="tutorial-version">
          {{ selected.version_note }}
        </p>
        <p class="tutorial-description">{{ selected.description }}</p>
        <p v-if="playbackError" role="alert" class="form-error">
          {{ playbackError }}
        </p>
      </div>
    </section>
    <p v-else class="notice">
      {{
        en
          ? "New tutorials will appear here when published."
          : "新的教学影片发布后会显示在这里。"
      }}
    </p>
    <section
      v-if="videos.length"
      class="tutorial-grid"
      :aria-label="en ? 'All tutorials' : '全部教学'"
    >
      <article
        v-for="video in videos"
        :key="video.id"
        class="tutorial-card"
        :class="{ selected: selected?.id === video.id }"
      >
        <button
          class="tutorial-cover"
          @click="choose(video)"
          :aria-label="`${en ? 'Play' : '播放'} ${video.title}`"
        >
          <img
            :src="video.poster_url"
            :alt="video.title"
            width="1280"
            height="720"
            loading="lazy"
          />
          <span class="tutorial-play" aria-hidden="true">▶</span>
        </button>
        <div>
          <h3>{{ video.title }}</h3>
          <p v-if="video.version_note">{{ video.version_note }}</p>
          <button class="button" @click="choose(video)">
            {{
              selected?.id === video.id
                ? en
                  ? "Play this episode"
                  : "播放本集"
                : en
                  ? "Watch episode"
                  : "观看本集"
            }}
            →
          </button>
        </div>
      </article>
    </section>
    <button v-if="more" class="button" :disabled="loading" @click="loadMore">
      {{ en ? "More tutorials" : "更多教学" }}
    </button>
    <p v-if="loadError" role="alert" class="form-error">{{ loadError }}</p>
  </main>
</template>
<style scoped>
.tutorial-page {
  padding-bottom: 80px;
}
.tutorial-languages {
  display: flex;
  gap: 10px;
  margin: 0 0 28px;
}
.tutorial-languages a {
  padding: 8px 18px;
  border: 1px solid var(--line);
  border-radius: 999px;
}
.tutorial-languages [aria-current] {
  background: var(--text);
  color: var(--bg);
}
.tutorial-watch {
  scroll-margin-top: 100px;
  border: 1px solid var(--line);
  border-radius: 18px;
  overflow: hidden;
  background: var(--surface);
}
.tutorial-watch video {
  display: block;
  width: 100%;
  aspect-ratio: 16/9;
  background: #111;
}
.tutorial-caption {
  padding: 28px;
}
.tutorial-caption h2 {
  font-size: clamp(22px, 3vw, 34px);
  margin: 0 0 12px;
}
.tutorial-version {
  display: inline-block;
  padding: 6px 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  color: var(--muted);
}
.tutorial-description {
  white-space: pre-line;
  max-width: 80ch;
  line-height: 1.8;
}
.tutorial-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 22px;
  margin: 32px 0;
}
.tutorial-card {
  min-width: 0;
  overflow: hidden;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--surface);
}
.tutorial-card.selected {
  border-color: var(--accent);
}
.tutorial-card > div {
  padding: 20px;
}
.tutorial-card h3 {
  font-size: 18px;
  line-height: 1.5;
  margin: 0 0 10px;
}
.tutorial-card p {
  color: var(--muted);
  font-size: 13px;
}
.tutorial-cover {
  position: relative;
  display: block;
  width: 100%;
  padding: 0;
  border: 0;
  cursor: pointer;
  background: #111;
}
.tutorial-cover img {
  display: block;
  width: 100%;
  height: auto;
  aspect-ratio: 16/9;
  object-fit: cover;
}
.tutorial-play {
  position: absolute;
  right: 14px;
  bottom: 12px;
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: #17332be8;
  color: white;
}
@media (max-width: 850px) {
  .tutorial-grid {
    grid-template-columns: 1fr;
  }
  .tutorial-card {
    display: grid;
    grid-template-columns: minmax(160px, 36%) 1fr;
    align-items: center;
  }
}
@media (max-width: 520px) {
  .tutorial-card {
    display: block;
  }
  .tutorial-caption {
    padding: 18px;
  }
}
</style>
