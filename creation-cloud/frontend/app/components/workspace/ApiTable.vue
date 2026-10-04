<script setup lang="ts">
import type { CloudPage } from "~/types/account";
export type Row = Record<string, unknown>;
const props = defineProps<{
  endpoint: string;
  columns: { key: string; label: string }[];
  array?: boolean;
}>();
const { t, en } = useLocale();
const api = useCloudApi();
const page = ref(1);
const items = ref<Row[]>([]);
const total = ref(0);
const loading = ref(false);
const error = ref("");
let loadRevision = 0;
async function load() {
  const revision = ++loadRevision;
  loading.value = true;
  error.value = "";
  try {
    const query = `${props.endpoint.includes("?") ? "&" : "?"}page=${page.value}&size=20`;
    const data = await api<CloudPage<Row> | Row[]>(
      props.endpoint + (props.array ? "" : query),
    );
    if (revision !== loadRevision) return;
    items.value = Array.isArray(data) ? data : data.items;
    total.value = Array.isArray(data) ? data.length : data.total;
  } catch (cause) {
    if (revision === loadRevision) error.value = cloudError(cause, en.value);
  } finally {
    if (revision === loadRevision) loading.value = false;
  }
}
function display(value: unknown) {
  if (value === null || value === undefined) return "—";
  if (typeof value === "boolean") return value ? t("是", "Yes") : t("否", "No");
  return typeof value === "string" || typeof value === "number"
    ? String(value)
    : "—";
}
onMounted(load);
watch(
  [() => props.endpoint, page],
  ([endpoint, currentPage], [oldEndpoint]) => {
    if (endpoint !== oldEndpoint && currentPage !== 1) {
      page.value = 1;
      return;
    }
    void load();
  },
);
defineExpose({ load, items });
</script>
<template>
  <div class="api-list">
    <p v-if="error" role="alert" class="form-error">{{ error }}</p>
    <div class="table-toolbar">
      <span>{{ t("记录", "Records") }} · {{ total }}</span
      ><button class="button" :disabled="loading" @click="load">
        {{ t("刷新", "Refresh") }}
      </button>
    </div>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th v-for="column in columns" :key="column.key">
              {{ column.label }}
            </th>
            <th v-if="$slots.actions">{{ t("操作", "Actions") }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(item, index) in items"
            :key="
              String(item.id || item.session_id || item.account_id || index)
            "
          >
            <td v-for="column in columns" :key="column.key">
              <slot :name="column.key" :row="item">{{
                display(item[column.key])
              }}</slot>
            </td>
            <td v-if="$slots.actions">
              <slot name="actions" :row="item" :reload="load" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <p v-if="!items.length && !loading && !error" class="empty-state">
      {{ t("暂无记录。", "No records.") }}
    </p>
    <p v-if="loading" role="status">{{ t("加载中…", "Loading…") }}</p>
    <div v-if="!array" class="pagination">
      <button :disabled="loading || page === 1" @click="page--">
        {{ t("上一页", "Previous") }}</button
      ><span>{{ page }}</span
      ><button :disabled="loading || page * 20 >= total" @click="page++">
        {{ t("下一页", "Next") }}
      </button>
    </div>
  </div>
</template>
