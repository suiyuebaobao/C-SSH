import type { ResourceConfig, FormField } from "~/types/forms";
const languages: FormField["options"] = [
  { value: "zh-CN", label: ["中文", "Chinese"] },
  { value: "en", label: ["英文", "English"] },
];
export const announcements: ResourceConfig = {
  title: ["公告", "Announcements"],
  endpoint: "/api/v1/admin/announcements",
  revision: true,
  columns: [
    { key: "title_zh_cn", label: ["中文标题", "Chinese title"] },
    { key: "title_en", label: ["英文标题", "English title"] },
    { key: "status", label: ["状态", "Status"] },
    { key: "priority", label: ["优先级", "Priority"] },
  ],
  fields: [
    {
      key: "title_zh_cn",
      label: ["中文标题", "Chinese title"],
      required: true,
    },
    {
      key: "body_zh_cn",
      label: ["中文正文", "Chinese content"],
      kind: "textarea",
      required: true,
    },
    { key: "title_en", label: ["英文标题", "English title"], required: true },
    {
      key: "body_en",
      label: ["英文正文", "English content"],
      kind: "textarea",
      required: true,
    },
    {
      key: "priority",
      label: ["优先级", "Priority"],
      kind: "select",
      default: "normal",
      options: [
        { value: "normal", label: ["一般", "Normal"] },
        { value: "important", label: ["重要", "Important"] },
        { value: "critical", label: ["紧急", "Critical"] },
      ],
    },
  ],
};
export const seo: ResourceConfig = {
  title: ["搜索优化", "Search topics"],
  endpoint: "/api/v1/admin/seo",
  array: true,
  columns: [
    { key: "phrase", label: ["关键词", "Phrase"] },
    { key: "locale", label: ["语言", "Language"] },
    { key: "enabled", label: ["启用", "Enabled"] },
    { key: "sort_order", label: ["顺序", "Order"] },
  ],
  fields: [
    {
      key: "locale",
      label: ["语言", "Language"],
      kind: "select",
      options: languages,
      default: "zh-CN",
    },
    { key: "phrase", label: ["关键词", "Phrase"], required: true },
    { key: "sort_order", label: ["顺序", "Order"], kind: "number", default: 0 },
    {
      key: "enabled",
      label: ["启用", "Enabled"],
      kind: "checkbox",
      default: true,
    },
  ],
};
export const users: ResourceConfig = {
  title: ["用户", "Users"],
  endpoint: "/api/v1/admin/users",
  search: true,
  columns: [
    { key: "display_name", label: ["名称", "Name"] },
    { key: "masked_email", label: ["邮箱", "Email"] },
    { key: "role", label: ["角色", "Role"] },
    { key: "status", label: ["状态", "Status"] },
    { key: "host_count", label: ["主机数", "Hosts"] },
  ],
  fields: [
    {
      key: "email",
      label: ["邮箱", "Email"],
      kind: "email",
      required: true,
      createOnly: true,
    },
    {
      key: "email",
      label: ["新邮箱（留空保留）", "New email (leave blank to keep)"],
      kind: "email",
      editOnly: true,
    },
    { key: "display_name", label: ["名称", "Name"], required: true },
    {
      key: "password",
      label: ["密码", "Password"],
      kind: "password",
      required: true,
      createOnly: true,
    },
    {
      key: "new_password",
      label: ["新密码（留空保留）", "New password (leave blank to keep)"],
      kind: "password",
      editOnly: true,
    },
    {
      key: "admin_login_name",
      label: ["管理员登录名", "Administrator login name"],
    },
    {
      key: "clear_admin_login_name",
      label: ["清除管理员登录名", "Clear administrator login name"],
      kind: "checkbox",
      editOnly: true,
    },
    {
      key: "role",
      label: ["角色", "Role"],
      kind: "select",
      default: "user",
      options: [
        { value: "user", label: ["用户", "User"] },
        { value: "admin", label: ["管理员", "Admin"] },
      ],
    },
    {
      key: "status",
      label: ["状态", "Status"],
      kind: "select",
      default: "active",
      options: [
        { value: "active", label: ["正常", "Active"] },
        { value: "disabled", label: ["停用", "Disabled"] },
      ],
    },
  ],
};
export const releases: ResourceConfig = {
  title: ["版本管理", "Releases"],
  endpoint: "/api/v1/admin/releases",
  columns: [
    { key: "version", label: ["版本", "Version"] },
    { key: "revision", label: ["修订", "Revision"] },
    { key: "channel", label: ["渠道", "Channel"] },
    { key: "status", label: ["状态", "Status"] },
    { key: "published_at", label: ["发布时间", "Published"] },
  ],
  fields: [
    {
      key: "version",
      label: ["版本号", "Version"],
      required: true,
      createOnly: true,
    },
    {
      key: "channel",
      label: ["渠道", "Channel"],
      kind: "select",
      default: "stable",
      createOnly: true,
      options: ["stable", "beta", "nightly"].map((value) => ({
        value,
        label: [value, value],
      })),
    },
    { key: "title_zh", label: ["中文标题", "Chinese title"], required: true },
    { key: "title_en", label: ["英文标题", "English title"], required: true },
    {
      key: "notes_zh",
      label: ["中文更新说明", "Chinese release notes"],
      kind: "textarea",
      required: true,
    },
    {
      key: "notes_en",
      label: ["英文更新说明", "English release notes"],
      kind: "textarea",
      required: true,
    },
    {
      key: "status",
      label: ["状态", "Status"],
      kind: "select",
      editOnly: true,
      options: [
        { value: "draft", label: ["草稿", "Draft"] },
        { value: "validating", label: ["待发布", "Validating"] },
      ],
    },
  ],
};
