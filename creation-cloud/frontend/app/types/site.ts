export interface Link {
  label: string;
  href: string;
}
export interface Item {
  badge: string;
  title: string;
  body: string;
  meta?: string;
}
export interface Section {
  anchor: string;
  title: string;
  lead: string;
  items: Item[];
}
export interface Faq {
  question: string;
  answer: string;
}
export interface Shell {
  locale: "zh-CN" | "en";
  html_lang: string;
  brand: string;
  brand_note: string;
  home_href: string;
  skip_label: string;
  menu_label: string;
  navigation: Link[];
  console_link: Link;
  login_link: Link;
  language_label: string;
  language_href: string;
  github_repository: Link;
  footer_summary: string;
  footer_motto: string;
  footer_note: string;
  footer_navigation: Link[];
}
export interface ContactWidget {
  title: string;
  note: string;
  pending_label: string;
  open_label: string;
  close_label: string;
}
export interface Home {
  qr_widget: ContactWidget;
  status_note: string;
  hero_blueprint_label: string;
  platform_label: string;
  platforms: {
    name: string;
    detail: string;
    state: string;
    planned: boolean;
  }[];
  sections: Section[];
  faq_heading: string;
  faq_lead: string;
  faqs: Faq[];
  seo_heading: string;
  final_heading: string;
  final_lead: string;
  media_slot: string | null;
}
export interface Documentation {
  index_label: string;
  search_label: string;
  search_placeholder: string;
  search_help: string;
  search_empty: string;
  status: { label: string; title: string; body: string };
  groups: {
    title: string;
    links: { anchor: string; code: string; title: string }[];
  }[];
  sections: (Omit<Section, "items"> & {
    code: string;
    items: (Item & {
      command: string;
      has_command: boolean;
      caution: boolean;
    })[];
  })[];
  tutorials: {
    index_label: string;
    prerequisites_label: string;
    steps_label: string;
    verification_label: string;
    agent_boundary_label: string;
    items: {
      anchor: string;
      sequence: string;
      title: string;
      summary: string;
      prerequisites: string[];
      steps: { label: string; title: string; instruction: string }[];
      verification: string;
      agent_boundary: string;
    }[];
  };
  screenshot: {
    title: string;
    lead: string;
    src: string;
    alt: string;
    caption: string;
    width: number;
    height: number;
  };
  final_title: string;
  final_body: string;
}
export interface Page {
  id: string;
  heading: string;
  eyebrow: string;
  lead: string;
  meta_title: string;
  meta_description: string;
  actions: Link[];
  sections: Section[];
  faqs: Faq[];
  home_page: Home | null;
  documentation_page: Documentation | null;
}
export interface Seo {
  robots: string;
  canonical_url: string | null;
  alternate_zh_url: string | null;
  alternate_en_url: string | null;
  alternate_default_url: string | null;
  open_graph_title: string | null;
  open_graph_description: string | null;
  open_graph_url: string | null;
  open_graph_locale: string | null;
  open_graph_alternate_locale: string | null;
  json_ld: string | null;
  google_site_verification: string | null;
  baidu_site_verification: string | null;
  keywords: string | null;
}
export interface Asset {
  architecture: string;
  package_kind: string;
  download_href: string | null;
}
export interface Platform {
  name: string;
  assets: Asset[];
}
export type NotesBlock =
  | { kind: "heading" | "paragraph"; content: string }
  | { kind: "bullets"; content: string[] };
export interface Release {
  version: string;
  channel: string;
  title: string;
  notes_blocks: NotesBlock[];
  published_at_iso: string;
  published_at_label: string;
  platforms: Platform[];
  manual_platforms: (Platform & { note: string })[];
  manual_notice: { title: string; body: string; link: Link } | null;
}
export interface SiteDocument {
  view: { shell: Shell; page: Page };
  seo: Seo;
  catalog: { releases: Release[]; latest: Release | null } | null;
  topics: string[];
}
