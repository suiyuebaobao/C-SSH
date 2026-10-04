import { defineStore } from "pinia";

// 跨导航组件的临时UI状态；不持久化用户资料、会话或服务器事实。
export const useNavigationStore = defineStore("navigation", () => {
  const menuOpen = ref(false);
  function close() {
    menuOpen.value = false;
  }
  function toggle() {
    menuOpen.value = !menuOpen.value;
  }
  return { menuOpen, close, toggle };
});
