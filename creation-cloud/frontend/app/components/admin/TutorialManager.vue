<script setup lang="ts">
const { t } = useLocale();
const {
  items,
  selected,
  preview,
  busy,
  reviewRequired,
  error,
  notice,
  offset,
  hasMore,
  video,
  poster,
  formKey,
  metadata,
  action,
  refresh,
  reset,
  edit,
  save,
  confirmAction,
  stateLabel,
} = useTutorialManager();
const formSection = useTemplateRef<HTMLElement>("formSection");
const dialog = useTemplateRef<HTMLDialogElement>("confirmation");
watch(action, async (value) => {
  await nextTick();
  if (value) dialog.value?.showModal();
  else dialog.value?.close();
});
</script>
<template>
  <h1>{{ t("教学视频", "Video tutorials") }}</h1>
  <p>
    {{
      t(
        "上传影片与封面，预览后发布到官网。",
        "Upload a video and cover, preview it, then publish it on the website.",
      )
    }}
  </p>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <p v-if="notice" role="status" class="notice">{{ notice }}</p>
  <p v-if="reviewRequired" role="status" class="notice">
    {{
      t(
        "请先刷新列表核对结果。刷新会结束当前编辑，再从最新列表选择影片。",
        "Refresh the library to review the result. This ends the current edit; select a video again from the latest library.",
      )
    }}
  </p>
  <section ref="formSection" class="card">
    <h2>
      {{
        selected
          ? t("编辑影片说明", "Edit video details")
          : t("添加教学影片", "Add a tutorial")
      }}
    </h2>
    <form :key="formKey" @submit.prevent="save">
      <fieldset :disabled="busy || reviewRequired" class="tutorial-fields">
        <label
          >{{ t("标题", "Title")
          }}<input v-model="metadata.title" maxlength="160" required
        /></label>
        <div class="tutorial-options">
          <label
            >{{ t("语言", "Language")
            }}<select v-model="metadata.locale">
              <option value="zh-CN">中文</option>
              <option value="en">English</option>
            </select></label
          >
          <label
            >{{ t("显示顺序", "Display order")
            }}<input
              v-model.number="metadata.sort_order"
              type="number"
              min="0"
              max="9999"
              required
          /></label>
        </div>
        <label
          >{{ t("录制版本说明（可选）", "Recording version note (optional)")
          }}<input
            v-model="metadata.version_note"
            maxlength="160"
            :placeholder="
              t(
                '例如：0.9.4 开发版；iOS 尚未公开发布',
                'For example: 0.9.4 development build; iOS not released',
              )
            "
        /></label>
        <label
          >{{ t("介绍与章节", "Description and chapters")
          }}<textarea
            v-model="metadata.description"
            maxlength="4000"
            rows="5"
          />
        </label>
        <div v-if="!selected" class="tutorial-options">
          <label
            >{{ t("MP4 影片（不超过200MiB）", "MP4 video (up to 200 MiB)")
            }}<input
              type="file"
              accept="video/mp4,.mp4"
              required
              @change="
                video = ($event.target as HTMLInputElement).files?.[0] || null
              "
          /></label>
          <label
            >{{
              t("PNG／JPEG 封面（不超过3MiB）", "PNG/JPEG cover (up to 3 MiB)")
            }}<input
              type="file"
              accept="image/png,image/jpeg"
              required
              @change="
                poster = ($event.target as HTMLInputElement).files?.[0] || null
              "
          /></label>
        </div>
        <p v-else class="muted">
          {{
            t(
              "影片文件保持原样；替换影片请新增记录后撤下旧片。",
              "The uploaded video remains unchanged. Add a new record to replace a video, then hide the old one.",
            )
          }}
        </p>
        <div class="tutorial-actions">
          <button class="button primary">
            {{
              busy
                ? t("正在保存…", "Saving…")
                : selected
                  ? t("保存说明", "Save details")
                  : t("上传并保存草稿", "Upload draft")
            }}
          </button>
          <button v-if="selected" type="button" class="button" @click="reset">
            {{ t("取消编辑", "Cancel editing") }}
          </button>
        </div>
      </fieldset>
    </form>
  </section>
  <section
    v-if="preview && preview.state !== 'deleted'"
    class="card tutorial-preview"
  >
    <h2>{{ t("预览", "Preview") }} · {{ preview.title }}</h2>
    <video
      :key="preview.id"
      :src="preview.video_url"
      :poster="preview.poster_url"
      :aria-label="preview.title"
      controls
      playsinline
      preload="none"
    />
    <button class="button" @click="preview = null">
      {{ t("关闭预览", "Close preview") }}
    </button>
  </section>
  <section class="card">
    <div class="tutorial-actions">
      <h2>{{ t("影片列表", "Video library") }}</h2>
      <button class="button" :disabled="busy" @click="refresh()">
        {{ t("刷新", "Refresh") }}
      </button>
    </div>
    <p v-if="!items.length && !busy && !error">
      {{ t("还没有教学影片。", "No tutorials yet.") }}
    </p>
    <article v-for="row in items" :key="row.id" class="tutorial-admin-row">
      <img
        v-if="row.state !== 'deleted'"
        :src="row.poster_url"
        :alt="row.title"
        width="160"
        height="90"
      />
      <div class="tutorial-admin-details">
        <strong>{{ row.title }}</strong>
        <p>
          {{ row.locale === "en" ? "English" : "中文" }} ·
          {{ stateLabel(row) }} ·
          {{ (row.video_bytes / 1048576).toFixed(1) }} MiB
        </p>
      </div>
      <div class="tutorial-actions">
        <button
          v-if="row.state !== 'deleted'"
          class="button"
          :disabled="busy"
          @click="preview = row"
        >
          {{ t("预览", "Preview") }}
        </button>
        <button
          v-if="row.state !== 'deleted'"
          class="button"
          :disabled="busy || reviewRequired"
          @click="
            edit(row);
            formSection?.scrollIntoView({ behavior: 'smooth' });
          "
        >
          {{ t("编辑", "Edit") }}
        </button>
        <button
          v-if="row.state === 'draft' || row.state === 'hidden'"
          class="button primary"
          :disabled="busy || reviewRequired"
          @click="action = { kind: 'publish', row }"
        >
          {{ t("发布", "Publish") }}
        </button>
        <button
          v-if="row.state === 'published'"
          class="button"
          :disabled="busy || reviewRequired"
          @click="action = { kind: 'hide', row }"
        >
          {{ t("撤下", "Hide") }}
        </button>
        <button
          class="button danger"
          :disabled="busy || reviewRequired"
          @click="action = { kind: 'delete', row }"
        >
          {{
            row.cleanup_pending
              ? t("继续清理", "Finish cleanup")
              : t("删除", "Delete")
          }}
        </button>
      </div>
    </article>
    <div class="tutorial-actions">
      <button
        v-if="offset > 0"
        class="button"
        :disabled="busy"
        @click="refresh(Math.max(0, offset - 100))"
      >
        {{ t("上一页", "Previous") }}</button
      ><button
        v-if="hasMore"
        class="button"
        :disabled="busy"
        @click="refresh(offset + 100)"
      >
        {{ t("下一页", "Next") }}
      </button>
    </div>
  </section>
  <dialog
    ref="confirmation"
    class="tutorial-confirm"
    @cancel.prevent="!busy && (action = null)"
  >
    <template v-if="action">
      <h2>
        {{
          action.kind === "delete"
            ? t("确认删除影片", "Delete this video?")
            : action.kind === "publish"
              ? t("确认发布影片", "Publish this video?")
              : t("确认撤下影片", "Hide this video?")
        }}
      </h2>
      <p>{{ action.row.title }}</p>
      <p>
        {{
          action.kind === "delete"
            ? t(
                "影片会从官网移除，视频和封面文件将被删除。",
                "The video will be removed from the website and its video and cover files deleted.",
              )
            : action.kind === "publish"
              ? t(
                  "发布后，任何人都可以在官网观看。",
                  "After publishing, anyone can watch it on the website.",
                )
              : t(
                  "撤下后官网停止播放，文件保留，可再次发布。",
                  "The video will stop playing publicly. Its files remain available for republishing.",
                )
        }}
      </p>
      <div class="tutorial-actions">
        <button class="button" :disabled="busy" @click="action = null">
          {{ t("取消", "Cancel") }}</button
        ><button class="button primary" :disabled="busy" @click="confirmAction">
          {{ busy ? t("处理中…", "Working…") : t("确认", "Confirm") }}
        </button>
      </div>
    </template>
  </dialog>
</template>
<style scoped>
.tutorial-fields {
  margin: 0;
  padding: 0;
  border: 0;
}
.tutorial-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
.tutorial-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
}
.tutorial-actions h2 {
  margin-right: auto;
}
.tutorial-preview video {
  display: block;
  width: min(100%, 900px);
  aspect-ratio: 16/9;
  margin-bottom: 18px;
  background: #111;
}
.tutorial-admin-row {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 20px 0;
  border-top: 1px solid var(--line);
}
.tutorial-admin-row img {
  object-fit: cover;
  flex-shrink: 0;
  border-radius: 8px;
}
.tutorial-admin-details {
  flex: 1;
  min-width: 0;
  overflow-wrap: anywhere;
}
.tutorial-admin-details p {
  color: var(--muted);
  font-size: 13px;
}
.tutorial-confirm {
  width: min(540px, calc(100vw - 32px));
  padding: 28px;
  border: 1px solid var(--line);
  border-radius: 16px;
  color: var(--text);
  background: var(--surface);
}
.tutorial-confirm::backdrop {
  background: #0009;
}
@media (max-width: 800px) {
  .tutorial-admin-row {
    flex-wrap: wrap;
  }
  .tutorial-options {
    grid-template-columns: 1fr;
    gap: 0;
  }
}
</style>
