export interface TutorialMetadata {
  locale: "zh-CN" | "en";
  title: string;
  description: string;
  sort_order: number;
  version_note: string;
}

export interface TutorialVideo extends TutorialMetadata {
  id: string;
  video_url: string;
  poster_url: string;
}

export interface AdminTutorial extends TutorialVideo {
  state: "draft" | "published" | "hidden" | "deleted";
  revision: number;
  video_bytes: number;
  updated_at: string;
  cleanup_pending: boolean;
}
