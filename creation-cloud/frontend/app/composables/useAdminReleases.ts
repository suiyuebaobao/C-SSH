import type { AdminRelease } from "~/types/releases";
import type { CloudPage } from "~/types/account";
export function useAdminReleases() {
  const api = useCloudApi();
  return async () => {
    const first = await api<CloudPage<AdminRelease>>(
      "/api/v1/admin/releases?page=1&size=100",
    );
    const result = [...first.items];
    // 冻结首次总页数，翻页不依赖会持续变动的集合长度。
    for (let page = 2; page <= Math.ceil(first.total / 100); page++) {
      const next = await api<CloudPage<AdminRelease>>(
        `/api/v1/admin/releases?page=${page}&size=100`,
      );
      result.push(...next.items);
    }
    return result;
  };
}
