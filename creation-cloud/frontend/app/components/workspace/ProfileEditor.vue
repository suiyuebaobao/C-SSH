<script setup lang="ts">
import type { Profile } from "~/types/account";
const { t, en } = useLocale();
const session = useSessionStore();
const api = useCloudApi();
const profile = ref<Profile | null>(null);
const currentPassword = ref("");
const newPassword = ref("");
const busy = ref(false);
const error = ref("");
const notice = ref("");
onMounted(async () => {
  try {
    profile.value = await api(`/api/v1/users/${session.current!.account_id}`);
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  }
});
async function save(password: boolean) {
  if (busy.value) return;
  busy.value = true;
  error.value = "";
  notice.value = "";
  try {
    if (password) {
      await api("/api/v1/auth/change-password", "POST", {
        current_password: currentPassword.value,
        new_password: newPassword.value,
        revoke_other_sessions: true,
      });
      await session.refresh();
    } else if (profile.value) {
      profile.value = await api(
        `/api/v1/users/${session.current!.account_id}`,
        "PATCH",
        {
          display_name: profile.value.display_name,
          locale: profile.value.locale,
        },
      );
    }
    notice.value = t("已保存。", "Saved.");
  } catch (cause) {
    error.value = cloudError(cause, en.value);
  } finally {
    currentPassword.value = "";
    newPassword.value = "";
    busy.value = false;
  }
}
</script>
<template>
  <h1>{{ t("账号与安全", "Account & security") }}</h1>
  <p v-if="error" role="alert" class="form-error">{{ error }}</p>
  <p v-if="notice" role="status" class="form-success">{{ notice }}</p>
  <section v-if="profile" class="card">
    <h2>{{ t("个人资料", "Profile") }}</h2>
    <form @submit.prevent="save(false)">
      <label
        >{{ t("名称", "Display name")
        }}<input
          v-model="profile.display_name"
          maxlength="80"
          required /></label
      ><label
        >{{ t("语言", "Language")
        }}<select v-model="profile.locale">
          <option value="zh-CN">中文</option>
          <option value="en">English</option>
        </select></label
      ><button class="button primary" :disabled="busy">
        {{ t("保存", "Save") }}
      </button>
    </form>
  </section>
  <section class="card">
    <h2>{{ t("修改密码", "Change password") }}</h2>
    <p>
      {{
        t(
          "修改密码将撤销旧会话，不改变客户端数据保护密码。",
          "Changing your password revokes old sessions. Your client data-protection password is unchanged.",
        )
      }}
    </p>
    <form @submit.prevent="save(true)">
      <label
        >{{ t("当前密码", "Current password")
        }}<input
          v-model="currentPassword"
          type="password"
          autocomplete="current-password"
          required /></label
      ><label
        >{{ t("新密码", "New password")
        }}<input
          v-model="newPassword"
          type="password"
          autocomplete="new-password"
          required /></label
      ><button class="button primary" :disabled="busy">
        {{ t("修改密码", "Change password") }}
      </button>
    </form>
  </section>
</template>
