<script setup lang="ts">
import type { PolicySnapshot } from "~/types/releases";
const { t, en } = useLocale();
const api = useCloudApi();
const data = ref<PolicySnapshot | null>(null);
const enabled = ref(false),
  sha256 = ref(true),
  confirmed = ref(false),
  busy = ref(false);
const target = ref(""),
  error = ref(""),
  notice = ref("");
const choices = ref<Record<string, string>>({});
const versions = computed(() =>
  data.value
    ? [
        ...new Set([
          ...data.value.target_releases.map((item) => item.version),
          ...data.value.draft.forced_versions,
          ...data.value.draft.disabled_versions,
          ...data.value.draft.no_update_versions,
        ]),
      ]
    : [],
);
async function load() {
  error.value = "";
  confirmed.value = false;
  try {
    data.value = await api("/api/v1/admin/downloads/policy");
    const draft = data.value!.draft;
    enabled.value = draft.enabled;
    sha256.value = draft.sha256_enabled;
    target.value = draft.target_release_id ?? "";
    choices.value = Object.fromEntries(
      versions.value.map((version) => [
        version,
        draft.disabled_versions.includes(version)
          ? "closed"
          : draft.no_update_versions.includes(version)
            ? "none"
            : draft.forced_versions.includes(version)
              ? "forced"
              : "optional",
      ]),
    );
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
}
onMounted(load);
async function apply() {
  if (!data.value || busy.value || !confirmed.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    const selected = (choice: string) =>
      Object.entries(choices.value)
        .filter(([, value]) => value === choice)
        .map(([version]) => version);
    await api("/api/v1/admin/downloads/policy/apply", "POST", {
      expected_published_revision: data.value.published.revision,
      confirmation: "apply_update_policy",
      draft: {
        expected_revision: data.value.draft.revision,
        enabled: enabled.value,
        target_release_id: target.value || null,
        sha256_enabled: sha256.value,
        forced_versions: selected("forced"),
        disabled_versions: selected("closed"),
        no_update_versions: selected("none"),
      },
    });
    notice.value = t("策略已发布。", "Policy published.");
    await load();
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="card">
    <h2>{{ t("更新设置", "Update policy") }}</h2>
    <p>
      {{
        t(
          "这里会直接影响客户端。保存并发布前，请核对目标版本和各旧版本的处理方式。",
          "These settings affect clients. Review the target and every version rule before publishing.",
        )
      }}
    </p>
    <p v-if="error" role="alert" class="form-error">{{ error }}</p>
    <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
    <form v-if="data" @submit.prevent="apply">
      <p>
        {{ t("当前策略版本", "Published policy revision") }}:
        {{ data.published.revision }}
      </p>
      <label
        ><input v-model="enabled" type="checkbox" />{{
          t("启用更新策略", "Enable update policy")
        }}</label
      ><label
        >{{ t("目标版本", "Target release")
        }}<select v-model="target">
          <option value="">{{ t("不指定目标", "No target") }}</option>
          <option
            v-for="item in data.target_releases"
            :key="item.id"
            :value="item.id"
            :disabled="!item.eligible"
          >
            {{ `${item.version} · ${t("修订", "Revision")} ${item.revision}`
            }}{{ item.eligible ? "" : t("（未就绪）", " (not ready)") }}
          </option>
        </select></label
      ><label
        ><input v-model="sha256" type="checkbox" />{{
          t("核对当前安装身份", "Verify current installation identity")
        }}</label
      >
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>{{ t("旧版本", "Existing version") }}</th>
              <th>{{ t("处理方式", "Action") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="version in versions" :key="version">
              <td>{{ version }}</td>
              <td>
                <select v-model="choices[version]" :aria-label="version">
                  <option value="optional">
                    {{ t("可选更新", "Optional update") }}
                  </option>
                  <option value="forced">
                    {{ t("强制更新", "Required update") }}
                  </option>
                  <option value="none">{{ t("无需更新", "No update") }}</option>
                  <option value="closed">
                    {{ t("关闭版本", "Disable version") }}
                  </option>
                </select>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <label
        ><input v-model="confirmed" type="checkbox" required />{{
          t(
            "已确认上述策略，将立即对客户端生效。",
            "I reviewed this policy; it will take effect for clients immediately.",
          )
        }}</label
      ><button class="button primary" :disabled="busy || !confirmed">
        {{ t("保存并发布", "Save and publish") }}
      </button>
    </form>
    <button class="button" :disabled="busy" @click="load">
      {{ t("刷新策略", "Refresh policy") }}
    </button>
  </section>
</template>
