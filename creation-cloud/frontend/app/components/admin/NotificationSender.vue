<script setup lang="ts">
const { t, en } = useLocale();
const api = useCloudApi();
const account = ref(""),
  resource = ref(""),
  expires = ref("");
const code = ref("security_review_required"),
  priority = ref("normal");
const busy = ref(false),
  confirmed = ref(false),
  error = ref(""),
  notice = ref("");
const codes = [
  ["security_review_required", "需要检查账号安全", "Review account security"],
  ["password_changed", "密码已变更", "Password changed"],
  ["device_revoked", "设备已撤销", "Device revoked"],
  ["session_revoked", "会话已撤销", "Session revoked"],
  ["sync_review_required", "需要检查同步", "Review sync"],
  ["sync_upload_completed", "上传完成", "Upload completed"],
  ["sync_download_completed", "下载完成", "Download completed"],
  ["sync_reset_completed", "云端数据已重置", "Cloud data reset"],
];
async function send() {
  if (busy.value || !confirmed.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    await api("/api/v1/admin/notifications", "POST", {
      account_id: account.value,
      kind: code.value.startsWith("sync_") ? "sync" : "account_security",
      code: code.value,
      priority: priority.value,
      resource_id: resource.value || null,
      expires_at: expires.value ? new Date(expires.value).toISOString() : null,
    });
    notice.value = t("已发送。", "Sent.");
    confirmed.value = false;
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="card">
    <h2>{{ t("账号通知", "Account notification") }}</h2>
    <p>
      {{
        t(
          "仅向指定账号发送固定类型通知；客户端公告界面不显示同步消息。",
          "Send a fixed notification to one account. Sync messages are not shown in the client announcement center.",
        )
      }}
    </p>
    <form @submit.prevent="send">
      <label
        >{{ t("账号标识", "Account ID")
        }}<input v-model="account" required /></label
      ><label
        >{{ t("通知类型", "Notification")
        }}<select v-model="code">
          <option v-for="item in codes" :key="item[0]" :value="item[0]">
            {{ en ? item[2] : item[1] }}
          </option>
        </select></label
      ><label
        >{{ t("优先级", "Priority")
        }}<select v-model="priority">
          <option value="normal">{{ t("一般", "Normal") }}</option>
          <option value="important">{{ t("重要", "Important") }}</option>
          <option value="critical">{{ t("紧急", "Critical") }}</option>
        </select></label
      ><label
        >{{ t("资源标识（可选）", "Resource ID (optional)")
        }}<input v-model="resource" /></label
      ><label
        >{{ t("到期时间（可选，本地时区）", "Expiry (optional, local time)")
        }}<input v-model="expires" type="datetime-local" /></label
      ><label
        ><input v-model="confirmed" type="checkbox" required />{{
          t(
            "确认账号和通知类型正确。",
            "I confirm the account and notification type.",
          )
        }}</label
      >
      <p v-if="error" role="alert" class="form-error">{{ error }}</p>
      <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
      <button class="button primary" :disabled="busy || !confirmed">
        {{ t("发送", "Send") }}
      </button>
    </form>
  </section>
</template>
