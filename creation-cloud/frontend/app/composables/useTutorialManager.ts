import type { AdminTutorial, TutorialMetadata } from "~/types/tutorials";

export function useTutorialManager() {
  const { en, t } = useLocale();
  const api = useCloudApi();
  const items = ref<AdminTutorial[]>([]);
  const selected = ref<AdminTutorial | null>(null);
  const preview = ref<AdminTutorial | null>(null);
  const busy = ref(false),
    error = ref(""),
    notice = ref("");
  const reviewRequired = ref(false);
  const offset = ref(0),
    hasMore = ref(false);
  const video = ref<File | null>(null),
    poster = ref<File | null>(null);
  const formKey = ref(0);
  const metadata = reactive<TutorialMetadata>({
    locale: "zh-CN",
    title: "",
    description: "",
    sort_order: 1,
    version_note: "",
  });
  const action = ref<{
    kind: "publish" | "hide" | "delete";
    row: AdminTutorial;
  } | null>(null);
  async function load() {
    items.value = await api<AdminTutorial[]>(
      `/api/v1/admin/tutorials?offset=${offset.value}`,
    );
    hasMore.value = items.value.length === 100;
  }
  async function refresh(nextOffset = offset.value) {
    if (busy.value) return;
    busy.value = true;
    error.value = "";
    notice.value = "";
    try {
      offset.value = nextOffset;
      await load();
      const current = items.value.find((row) => row.id === selected.value?.id);
      if (
        reviewRequired.value ||
        (selected.value && current?.revision !== selected.value.revision)
      ) {
        reset();
        notice.value = t(
          "已读取最新列表。请核对操作结果，再选择影片继续编辑。",
          "The latest library is loaded. Review the result, then select a video to continue editing.",
        );
      }
      if (preview.value)
        preview.value =
          items.value.find((row) => row.id === preview.value?.id) ?? null;
      reviewRequired.value = false;
    } catch (cause) {
      error.value = cloudError(cause, en.value);
    } finally {
      busy.value = false;
    }
  }
  function writeError(cause: unknown) {
    error.value = cloudError(cause, en.value);
    const status = (cause as { status?: number }).status;
    reviewRequired.value =
      !status || status === 404 || status === 409 || status >= 500;
  }
  async function loadAfterWrite(message: string) {
    try {
      await load();
      notice.value = message;
    } catch {
      reviewRequired.value = true;
      error.value = t(
        "操作已完成，但列表未能刷新。请刷新列表后继续。",
        "The operation completed, but the library could not be refreshed. Refresh it before continuing.",
      );
    }
  }
  function reset() {
    selected.value = null;
    video.value = null;
    poster.value = null;
    formKey.value++;
    Object.assign(metadata, {
      locale: "zh-CN",
      title: "",
      description: "",
      sort_order: 1,
      version_note: "",
    });
  }
  function edit(row: AdminTutorial) {
    selected.value = { ...row };
    preview.value = row;
    Object.assign(metadata, {
      locale: row.locale,
      title: row.title,
      description: row.description,
      sort_order: row.sort_order,
      version_note: row.version_note,
    });
    error.value = "";
    notice.value = "";
  }
  async function save() {
    if (busy.value || reviewRequired.value) return;
    error.value = "";
    notice.value = "";
    if (!selected.value && (!video.value || !poster.value)) {
      error.value = t(
        "请选择影片和封面。",
        "Choose a video and a cover image.",
      );
      return;
    }
    if (
      !selected.value &&
      (video.value!.size > 200 * 1024 * 1024 ||
        poster.value!.size > 3 * 1024 * 1024)
    ) {
      error.value = t(
        "影片上限200MiB，封面上限3MiB。",
        "Video limit: 200 MiB. Cover limit: 3 MiB.",
      );
      return;
    }
    busy.value = true;
    try {
      let row: AdminTutorial;
      if (selected.value) {
        row = await api<AdminTutorial>(
          `/api/v1/admin/tutorials/${selected.value.id}`,
          "PATCH",
          {
            expected_revision: selected.value.revision,
            metadata: { ...metadata },
          },
        );
      } else {
        const body = new FormData();
        body.set("metadata", JSON.stringify(metadata));
        body.set("video", video.value!);
        body.set("poster", poster.value!);
        row = await api<AdminTutorial>("/api/v1/admin/tutorials", "POST", body);
      }
      preview.value = row;
      reset();
      offset.value = 0;
      await loadAfterWrite(
        row.state === "draft"
          ? t(
              "草稿已保存，请预览后发布。",
              "Draft saved. Preview it before publishing.",
            )
          : t("说明已保存。", "Details saved."),
      );
    } catch (cause) {
      writeError(cause);
    } finally {
      busy.value = false;
    }
  }
  async function confirmAction() {
    const pending = action.value;
    if (!pending || busy.value || reviewRequired.value) return;
    busy.value = true;
    error.value = "";
    notice.value = "";
    try {
      const path = `/api/v1/admin/tutorials/${pending.row.id}${pending.kind === "delete" ? "" : `/${pending.kind}`}`;
      await api(path, pending.kind === "delete" ? "DELETE" : "POST", {
        expected_revision: pending.row.revision,
      });
      if (selected.value?.id === pending.row.id) reset();
      if (preview.value?.id === pending.row.id) preview.value = null;
      await loadAfterWrite(t("操作已完成。", "Operation completed."));
    } catch (cause) {
      writeError(cause);
    } finally {
      action.value = null;
      busy.value = false;
    }
  }
  const stateLabel = (row: AdminTutorial) => {
    if (row.cleanup_pending)
      return t("已删除，文件待清理", "Deleted; file cleanup pending");
    return {
      draft: t("草稿", "Draft"),
      published: t("已发布", "Published"),
      hidden: t("已撤下", "Hidden"),
      deleted: t("已删除", "Deleted"),
    }[row.state];
  };
  onMounted(() => refresh());
  return {
    items,
    selected,
    preview,
    busy,
    reviewRequired,
    error,
    notice,
    offset,
    hasMore,
    video,
    poster,
    formKey,
    metadata,
    action,
    refresh,
    reset,
    edit,
    save,
    confirmAction,
    stateLabel,
  };
}
