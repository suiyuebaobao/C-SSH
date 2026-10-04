<script setup lang="ts">
import type { ResourceConfig } from "~/types/forms";
import type { Row } from "~/components/workspace/ApiTable.vue";
const props = defineProps<{
  config: ResourceConfig;
  deletable?: boolean;
  readonlyPublished?: boolean;
}>();
const { en, t } = useLocale();
const api = useCloudApi();
const table = useTemplateRef("table");
const selected = ref<Row | null>(null);
const opened = ref(false);
const form = ref<Record<string, unknown>>({});
const busy = ref(false);
const error = ref("");
const searchInput = ref("");
const search = ref("");
const listEndpoint = computed(
  () =>
    props.config.endpoint +
    (search.value ? `?search=${encodeURIComponent(search.value)}` : ""),
);
const columns = computed(() =>
  props.config.columns.map((column) => ({
    key: column.key,
    label: column.label[en.value ? 1 : 0],
  })),
);
function edit(row: Row | null) {
  selected.value = row ? structuredClone(toRaw(row)) : null;
  form.value = Object.fromEntries(
    props.config.fields.map((field) => [
      field.key,
      field.kind === "password"
        ? ""
        : (row?.[field.key] ??
          field.default ??
          (field.kind === "checkbox" ? false : "")),
    ]),
  );
  error.value = "";
  opened.value = true;
}
async function save() {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  const body: Record<string, unknown> = {};
  for (const field of props.config.fields) {
    if (
      (selected.value && field.createOnly) ||
      (!selected.value && field.editOnly)
    )
      continue;
    const value = form.value[field.key];
    if (!field.required && (value === "" || value === undefined)) continue;
    body[field.key] = value;
  }
  if (selected.value && props.config.revision)
    body.expected_revision = selected.value.revision;
  try {
    await api(
      `${props.config.endpoint}${selected.value ? `/${selected.value.id}` : ""}`,
      selected.value ? "PATCH" : "POST",
      body,
    );
    opened.value = false;
    form.value = {};
    await table.value?.load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
function locked(row: Row) {
  return (
    props.readonlyPublished &&
    ["published", "revoked", "hidden"].includes(String(row.status))
  );
}
</script>
<template>
  <h1>{{ config.title[en ? 1 : 0] }}</h1>
  <form
    v-if="config.search"
    class="filter-bar"
    @submit.prevent="search = searchInput.trim()"
  >
    <label
      >{{ t("搜索用户", "Search users")
      }}<input v-model="searchInput" type="search"
    /></label>
    <button class="button">{{ t("搜索", "Search") }}</button>
  </form>
  <button class="button primary" @click="edit(null)">
    {{ t("新增", "Create") }}</button
  ><WorkspaceApiTable
    ref="table"
    :endpoint="listEndpoint"
    :columns="columns"
    :array="config.array"
    ><template #actions="{ row, reload }"
      ><button v-if="!locked(row)" class="button" @click="edit(row)">
        {{ t("编辑", "Edit") }}</button
      ><slot
        name="actions"
        :row="row"
        :reload="reload" /><WorkspaceConfirmAction
        v-if="deletable && !locked(row)"
        :label="t('删除', 'Delete')"
        :message="
          t(
            '确认删除这条记录？此操作不能撤销。',
            'Delete this record? This cannot be undone.',
          )
        "
        :endpoint="`${config.endpoint}/${row.id}`"
        :body="
          config.revision ? { expected_revision: row.revision } : undefined
        "
        @done="reload" /></template></WorkspaceApiTable
  ><Teleport to="body"
    ><div v-if="opened" class="modal-overlay">
      <section
        class="modal card"
        role="dialog"
        aria-modal="true"
        :aria-label="config.title[en ? 1 : 0]"
      >
        <h2>{{ selected ? t("编辑", "Edit") : t("新增", "Create") }}</h2>
        <form @submit.prevent="save">
          <AdminFieldsEditor
            v-model="form"
            :fields="config.fields"
            :edit="!!selected"
          />
          <p v-if="error" role="alert" class="form-error">{{ error }}</p>
          <button
            class="button"
            type="button"
            :disabled="busy"
            @click="
              opened = false;
              form = {};
            "
          >
            {{ t("取消", "Cancel") }}</button
          ><button class="button primary" :disabled="busy">
            {{ t("保存", "Save") }}
          </button>
        </form>
      </section>
    </div></Teleport
  >
</template>
