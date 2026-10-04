<script setup lang="ts">
import type { AdminRelease, AdminAsset } from "~/types/releases";
const emit = defineEmits<{ done: [] }>();
const { t, en } = useLocale();
const api = useCloudApi();
const route = useRoute();
const listReleases = useAdminReleases();
const releases = ref<AdminRelease[]>([]);
const release = ref(""),
  platform = ref("windows"),
  architecture = ref("x86_64"),
  packageKind = ref("exe"),
  mode = ref("local");
const external = ref(""),
  fileName = ref(""),
  byteSize = ref<number | undefined>(),
  sha256 = ref("");
const file = ref<File | null>(null),
  signature = ref<File | null>(null);
const assets = ref<AdminAsset[]>([]);
const packages = computed(() => platform.value === "android" ? ["apk"] : ["exe", "zip"]);
watch(platform, (value) => {
  architecture.value = value === "android" ? "aarch64" : "x86_64";
  packageKind.value = packages.value[0]!;
  signature.value = null;
});
const busy = ref(false),
  error = ref(""),
  notice = ref("");
const existing = computed(() =>
  assets.value.some(
    (item) =>
      item.platform === platform.value &&
      item.architecture === architecture.value &&
      item.package_kind === packageKind.value,
  ),
);
onMounted(async () => {
  try {
    releases.value = (await listReleases()).filter((item) =>
      ["draft", "validating"].includes(item.status),
    );
    const requested = route.query.release_id;
    if (
      typeof requested === "string" &&
      releases.value.some((item) => item.id === requested)
    )
      release.value = requested;
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
watch(release, async (id) => {
  assets.value = [];
  if (!id) return;
  try {
    assets.value = await api(`/api/v1/admin/releases/${id}/assets`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
async function upload() {
  if (busy.value || !release.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    const body = new FormData();
    for (const [key, value] of Object.entries({
      release_id: release.value,
      platform: platform.value,
      architecture: architecture.value,
      package_kind: packageKind.value,
      source_mode: mode.value,
    }))
      body.set(key, value);
    if (mode.value === "local") {
      if (!file.value) throw new Error("Missing file");
      body.set("file", file.value);
    } else {
      body.set("external_url", external.value);
      if (!existing.value) {
        body.set("file_name", fileName.value);
        body.set("byte_size", String(byteSize.value ?? ""));
        body.set("sha256", sha256.value);
      }
    }
    if (platform.value === "windows" && signature.value)
      body.set("updater_signature", signature.value);
    await api("/api/v1/admin/downloads/new", "POST", body);
    notice.value = t("下载已登记。", "Download registered.");
    emit("done");
    assets.value = await api(`/api/v1/admin/releases/${release.value}/assets`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="card">
    <h2>{{ t("上传客户端／添加下载", "Upload client / add download") }}</h2>
    <form @submit.prevent="upload">
      <label
        >{{ t("发布版本", "Release")
        }}<select v-model="release" required>
          <option value="" disabled>
            {{
              t("选择草稿或待发布版本", "Select a draft or validating release")
            }}
          </option>
          <option v-for="item in releases" :key="item.id" :value="item.id">
            {{ item.version }}
          </option>
        </select></label
      ><label
        >{{ t("平台", "Platform")
        }}<select v-model="platform">
          <option
            v-for="value in ['windows', 'android']"
            :key="value"
            :value="value"
          >
            {{ value }}
          </option>
        </select></label
      ><label
        >{{ t("架构", "Architecture")
        }}<select v-model="architecture">
          <option v-if="platform === 'windows'" value="x86_64">x86_64</option>
          <option v-else value="aarch64">ARM64</option>
        </select></label
      ><label
        >{{ t("包格式", "Package")
        }}<select v-model="packageKind">
          <option
            v-for="value in packages"
            :key="value"
            :value="value"
          >
            {{ value }}
          </option>
        </select></label
      ><label
        >{{ t("下载方式", "Source")
        }}<select v-model="mode">
          <option value="local">{{ t("本地文件上传", "Upload file") }}</option>
          <option value="external">{{ t("HTTPS 外链", "HTTPS link") }}</option>
        </select></label
      ><label v-if="mode === 'local'"
        >{{ t("客户端文件", "Client file")
        }}<input
          type="file"
          required
          @change="
            file = ($event.target as HTMLInputElement).files?.[0] || null
          " /></label
      ><template v-else
        ><label
          >{{ t("HTTPS 地址", "HTTPS URL")
          }}<input
            v-model="external"
            type="url"
            required
            placeholder="https://example.com/client.zip"
        /></label>
        <details v-if="!existing" open>
          <summary>
            {{
              t(
                "首次登记外链文件的校验信息",
                "Integrity information for a new external file",
              )
            }}
          </summary>
          <label
            >{{ t("文件名", "File name")
            }}<input v-model="fileName" required /></label
          ><label
            >{{ t("字节数", "Size in bytes")
            }}<input
              v-model.number="byteSize"
              type="number"
              min="1"
              required /></label
          ><label
            >SHA256<input v-model="sha256" required pattern="[a-fA-F0-9]{64}"
          /></label></details></template
      ><label v-if="platform === 'windows'"
        >{{
          t(
            "更新签名文件（本地上传必需）",
            "Updater signature file (required for local upload)",
          )
        }}<input
          type="file"
          accept=".sig"
          :required="mode === 'local'"
          @change="
            signature = ($event.target as HTMLInputElement).files?.[0] || null
          "
      /></label>
      <p v-if="error" role="alert" class="form-error">{{ error }}</p>
      <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
      <button class="button primary" :disabled="busy">
        {{
          busy
            ? t("正在处理，请勿重复提交…", "Working. Do not resubmit…")
            : t("添加下载", "Add download")
        }}
      </button>
    </form>
  </section>
</template>
