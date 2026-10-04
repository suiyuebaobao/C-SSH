<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const fields = ref<Record<string, unknown>>({
  category: "bug",
  platform: "windows",
  title: "",
  description: "",
  app_version: "",
  redaction_confirmed: false,
});
const busy = ref(false);
const error = ref("");
const notice = ref("");
const list = useTemplateRef("list");
const selected = ref<Record<string, unknown> | null>(null);
const columns = computed(() => [
  { key: "title", label: t("标题", "Title") },
  { key: "platform", label: t("平台", "Platform") },
  { key: "status", label: t("状态", "Status") },
  { key: "created_at", label: t("提交时间", "Submitted") },
]);
async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    await api("/api/v1/feedback", "POST", {
      ...fields.value,
      app_version: fields.value.app_version || null,
    });
    fields.value.title = "";
    fields.value.description = "";
    fields.value.redaction_confirmed = false;
    notice.value = t("反馈已提交。", "Feedback submitted.");
    await list.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <h2>{{ t("反馈", "Feedback") }}</h2>
  <p>
    {{
      t(
        "也可以通过 GitHub Issues 提交公开问题。请勿包含凭据、真实主机地址或私人对话。",
        "You can also report public issues on GitHub. Do not include credentials, real host addresses or private conversations.",
      )
    }}
  </p>
  <SiteLink href="https://github.com/suiyuebaobao/C-SSH/issues" class="button"
    >GitHub Issues ↗</SiteLink
  >
  <form class="card" @submit.prevent="submit">
    <label
      >{{ t("类别", "Category")
      }}<select v-model="fields.category">
        <option
          v-for="value in ['bug', 'feature', 'docs', 'compatibility', 'other']"
          :key="value"
          :value="value"
        >
          {{ value }}
        </option>
      </select></label
    ><label
      >{{ t("平台", "Platform")
      }}<select v-model="fields.platform">
        <option
          v-for="value in [
            'windows',
            'macos',
            'linux',
            'android',
            'ios',
            'cloud',
            'agent',
            'other',
          ]"
          :key="value"
          :value="value"
        >
          {{ value }}
        </option>
      </select></label
    ><label
      >{{ t("客户端版本（可选）", "App version (optional)")
      }}<input v-model="fields.app_version" /></label
    ><label
      >{{ t("标题", "Title")
      }}<input v-model="fields.title" required maxlength="120" /></label
    ><label
      >{{ t("描述", "Description")
      }}<textarea
        :value="String(fields.description)"
        required
        maxlength="8000"
        @input="
          fields.description = ($event.target as HTMLTextAreaElement).value
        "
      /></label
    ><label
      ><input v-model="fields.redaction_confirmed" type="checkbox" required />{{
        t(
          "已检查内容，不包含秘密或私人数据。",
          "I checked that no secrets or private data are included.",
        )
      }}</label
    >
    <p v-if="error" role="alert" class="form-error">{{ error }}</p>
    <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
    <button class="button primary" :disabled="busy">
      {{ t("提交反馈", "Submit feedback") }}
    </button>
  </form>
  <h2>{{ t("我的反馈", "My feedback") }}</h2>
  <WorkspaceApiTable ref="list" endpoint="/api/v1/feedback" :columns="columns"
    ><template #actions="{ row }"
      ><button class="button" @click="selected = row">
        {{ t("查看", "View") }}
      </button></template
    ></WorkspaceApiTable
  >
  <section v-if="selected" class="card">
    <h2>{{ selected.title }}</h2>
    <p class="plain-text">{{ selected.description }}</p>
  </section>
</template>
