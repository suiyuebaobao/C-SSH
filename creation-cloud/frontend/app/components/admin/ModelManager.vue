<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const table = useTemplateRef("table");
const opened = ref(false);
const busy = ref(false);
const error = ref("");
const selected = ref<Record<string, unknown> | null>(null);
const name = ref(""),
  provider = ref(""),
  contextLength = ref(128000),
  enabled = ref(true);
const interfaces = ref<
  { api_format: string; base_url: string; model_name: string }[]
>([]);
const columns = computed(() => [
  { key: "name", label: t("名称", "Name") },
  { key: "provider", label: t("厂商", "Provider") },
  { key: "context_length", label: t("上下文", "Context length") },
  { key: "enabled", label: t("启用", "Enabled") },
]);
function edit(row: Record<string, unknown> | null) {
  selected.value = row ? structuredClone(toRaw(row)) : null;
  name.value = String(row?.name ?? "");
  provider.value = String(row?.provider ?? "");
  contextLength.value = Number(row?.context_length ?? 128000);
  enabled.value = Boolean(row?.enabled ?? true);
  interfaces.value = row
    ? (structuredClone(toRaw(row.interfaces)) as typeof interfaces.value)
    : [{ api_format: "openai_compatible", base_url: "", model_name: "" }];
  error.value = "";
  opened.value = true;
}
async function save() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  try {
    const body: Record<string, unknown> = {
      name: name.value,
      provider: provider.value,
      context_length: contextLength.value,
      enabled: enabled.value,
      interfaces: interfaces.value,
      reasoning_control: "unsupported",
    };
    if (selected.value) {
      body.expected_revision = selected.value.revision;
      for (const key of [
        "is_default",
        "sort_order",
        "default_parameters",
        "capability_tags",
      ])
        body[key] = selected.value[key];
    }
    await api(
      `/api/v1/admin/models${selected.value ? `/${selected.value.id}` : ""}`,
      selected.value ? "PATCH" : "POST",
      body,
    );
    opened.value = false;
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <h1>{{ t("全局模型目录", "Global model directory") }}</h1>
  <button class="button primary" @click="edit(null)">
    {{ t("新增模型", "Add model") }}</button
  ><WorkspaceApiTable
    ref="table"
    endpoint="/api/v1/admin/models"
    :columns="columns"
    ><template #actions="{ row, reload }"
      ><button class="button" @click="edit(row)">{{ t("编辑", "Edit") }}</button
      ><WorkspaceConfirmAction
        :label="t('删除', 'Delete')"
        :message="t('删除这项全局模型？', 'Delete this global model?')"
        :endpoint="`/api/v1/admin/models/${row.id}`"
        :body="{ expected_revision: row.revision }"
        @done="reload" /></template></WorkspaceApiTable
  ><Teleport to="body"
    ><div v-if="opened" class="modal-overlay">
      <section
        class="modal card"
        role="dialog"
        aria-modal="true"
        :aria-label="t('模型', 'Model')"
      >
        <h2>{{ t("模型", "Model") }}</h2>
        <form @submit.prevent="save">
          <label>{{ t("名称", "Name") }}<input v-model="name" required /></label
          ><label
            >{{ t("厂商", "Provider")
            }}<input v-model="provider" required /></label
          ><label
            >{{ t("上下文长度", "Context length")
            }}<input
              v-model.number="contextLength"
              type="number"
              min="1"
              required /></label
          ><label
            ><input v-model="enabled" type="checkbox" />{{
              t("启用", "Enabled")
            }}</label
          >
          <fieldset v-for="(item, index) in interfaces" :key="index">
            <legend>{{ t("接口", "Interface") }} {{ index + 1 }}</legend>
            <label
              >{{ t("协议", "Protocol")
              }}<select v-model="item.api_format">
                <option value="openai_compatible">OpenAI</option>
                <option value="responses_compatible">Responses</option>
                <option value="anthropic_compatible">Anthropic</option>
              </select></label
            ><label
              >Base URL<input
                v-model="item.base_url"
                type="url"
                required
                placeholder="https://example.com/v1" /></label
            ><label>Model ID<input v-model="item.model_name" required /></label
            ><button
              v-if="interfaces.length > 1"
              class="button"
              type="button"
              @click="interfaces.splice(index, 1)"
            >
              {{ t("移除接口", "Remove interface") }}
            </button>
          </fieldset>
          <button
            v-if="interfaces.length < 3"
            class="button"
            type="button"
            @click="
              interfaces.push({
                api_format: 'responses_compatible',
                base_url: '',
                model_name: '',
              })
            "
          >
            {{ t("增加接口", "Add interface") }}
          </button>
          <p v-if="error" class="form-error" role="alert">{{ error }}</p>
          <div class="actions">
            <button
              class="button"
              type="button"
              :disabled="busy"
              @click="opened = false"
            >
              {{ t("取消", "Cancel") }}</button
            ><button class="button primary" :disabled="busy">
              {{ t("保存", "Save") }}
            </button>
          </div>
        </form>
      </section>
    </div></Teleport
  >
</template>
