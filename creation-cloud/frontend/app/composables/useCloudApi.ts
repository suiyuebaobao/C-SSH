export function useCloudApi() {
  const session = useSessionStore();
  return async function request<T>(
    path: string,
    method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE" = "GET",
    body?: Record<string, unknown> | FormData,
  ): Promise<T> {
    if (import.meta.server) throw new Error("Private API is browser-only");
    if (
      !path.startsWith("/api/v1/") ||
      path.includes("://") ||
      path.includes("\\")
    )
      throw new Error("Invalid API path");
    const headers: Record<string, string> = {};
    if (method !== "GET" && session.current)
      headers["X-CSRF-Token"] = session.current.csrf_token;
    try {
      const result: unknown = await $fetch(path, {
        method,
        body,
        headers,
        credentials: "same-origin",
        cache: "no-store",
        retry: 0,
        timeout: body instanceof FormData ? 30 * 60 * 1000 : 30000,
      });
      return result as T;
    } catch (error) {
      if ((error as { status?: number }).status === 401) session.clear();
      throw error;
    }
  };
}

export function cloudError(error: unknown, en = false): string {
  const status = (error as { status?: number }).status;
  const messages: Record<number, [string, string]> = {
    400: ["输入无效，请检查后重试。", "Check the submitted fields."],
    401: [
      "认证失败或登录已失效。",
      "Authentication failed or the session expired.",
    ],
    403: [
      "没有权限，或页面验证已失效。请重新登录。",
      "Access denied or page verification expired. Sign in again.",
    ],
    409: [
      "内容已变化或操作冲突，请刷新后重新确认。",
      "The record changed or conflicts with this operation. Refresh and review it.",
    ],
    429: [
      "请求过于频繁，请稍后再试。",
      "Too many requests. Please try again later.",
    ],
  };
  return (
    messages[status ?? 0]?.[en ? 1 : 0] ??
    (en
      ? "The operation could not be confirmed. Refresh its state before retrying."
      : "操作结果未能确认。请刷新状态后再决定是否重试。")
  );
}
