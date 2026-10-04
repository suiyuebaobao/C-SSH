import { defineStore } from "pinia";
import type { Session } from "~/types/account";

export const useSessionStore = defineStore("session", () => {
  // 仅浏览器内存使用；禁止SSR读取Cookie、localStorage持久化或记录会话内容。
  const current = ref<Session | null>(null);
  const email = ref("");
  const challenge = ref("");
  function clear() {
    current.value = null;
    email.value = "";
    challenge.value = "";
  }
  async function refresh() {
    if (import.meta.server) return null;
    try {
      current.value = await $fetch<Session>("/api/v1/auth/session", {
        credentials: "same-origin",
        cache: "no-store",
        retry: 0,
      });
      return current.value;
    } catch (error) {
      if ((error as { status?: number }).status === 401) clear();
      throw error;
    }
  }
  return { current, email, challenge, clear, refresh };
});
