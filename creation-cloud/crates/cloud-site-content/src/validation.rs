use cloud_domain::{AppError, AppResult};
use cloud_site::{
    EditableHomePage, EditableLink, EditableSiteShell, Locale, SITE_CONTENT_SCHEMA_VERSION,
};
use serde_json::Value;
use url::Url;
use uuid::Uuid;

use crate::{SiteContentDocumentKey, SiteContentPayload, SiteContentRevision, SiteContentState};

const MAX_DOCUMENT_BYTES: usize = 128 * 1024;
const MAX_TEXT_CHARS: usize = 2_000;

pub(crate) fn id(value: Uuid) -> AppResult<Uuid> {
    if value.is_nil() {
        return Err(AppError::Validation("站点内容标识无效".into()));
    }
    Ok(value)
}

pub(crate) fn expected_revision(value: i64) -> AppResult<i64> {
    if value < 1 {
        return Err(AppError::Validation("expected_revision 必须大于零".into()));
    }
    Ok(value)
}

pub(crate) fn payload(
    key: SiteContentDocumentKey,
    locale: Locale,
    payload: SiteContentPayload,
) -> AppResult<SiteContentPayload> {
    if payload.key() != key {
        return Err(AppError::Validation("内容结构与文档类型不一致".into()));
    }
    let value = serde_json::to_value(&payload)
        .map_err(|_| AppError::Validation("站点内容无法序列化".into()))?;
    let bytes = serde_json::to_vec(&value)
        .map_err(|_| AppError::Validation("站点内容无法序列化".into()))?;
    if bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(AppError::Validation("站点内容超过 128 KiB 上限".into()));
    }
    inspect_value(&value, "$", 0)?;
    match &payload {
        SiteContentPayload::SiteShell(document) => validate_shell(document, locale)?,
        SiteContentPayload::Home(document) => validate_home(document, locale)?,
    }
    Ok(payload)
}

pub(crate) fn editable_draft(
    record: &SiteContentRevision,
    expected_revision: i64,
) -> AppResult<()> {
    if record.state != SiteContentState::Draft {
        return Err(AppError::Conflict("只有草稿可以修改或删除".into()));
    }
    compare_revision(record, expected_revision)
}

pub(crate) fn publishable(record: &SiteContentRevision, expected_revision: i64) -> AppResult<()> {
    if record.state != SiteContentState::Draft {
        return Err(AppError::Conflict("只有草稿可以发布".into()));
    }
    compare_revision(record, expected_revision)
}

pub(crate) fn revocable(record: &SiteContentRevision, expected_revision: i64) -> AppResult<()> {
    if record.state != SiteContentState::Published {
        return Err(AppError::Conflict("只有当前已发布内容可以撤销".into()));
    }
    compare_revision(record, expected_revision)
}

pub(crate) fn rollback_source(
    record: &SiteContentRevision,
    expected_revision: i64,
) -> AppResult<()> {
    if record.state == SiteContentState::Draft {
        return Err(AppError::Conflict("草稿不能作为历史回滚来源".into()));
    }
    compare_revision(record, expected_revision)
}

#[must_use]
pub(crate) fn field_count(payload: &SiteContentPayload) -> usize {
    serde_json::to_value(payload)
        .map(|value| count_leaves(&value))
        .unwrap_or(0)
}

fn compare_revision(record: &SiteContentRevision, expected: i64) -> AppResult<()> {
    let expected = expected_revision(expected)?;
    if record.revision != expected {
        return Err(AppError::Conflict(
            "站点内容已被其他管理员更新，请刷新后重试".into(),
        ));
    }
    Ok(())
}

fn validate_shell(document: &EditableSiteShell, locale: Locale) -> AppResult<()> {
    if document.schema_version != SITE_CONTENT_SCHEMA_VERSION {
        return Err(AppError::Validation(
            "站点壳内容 schema_version 无效".into(),
        ));
    }
    let baseline = cloud_site::compiled_site_shell(locale);
    exact_len(
        document.navigation.len(),
        baseline.navigation.len(),
        "页头导航",
    )?;
    exact_len(
        document.footer_navigation.len(),
        baseline.footer_navigation.len(),
        "页脚导航",
    )?;
    required(&document.brand_note, "品牌副标题", 120)?;
    required(&document.skip_label, "跳转标签", 80)?;
    required(&document.menu_label, "菜单标签", 80)?;
    required(&document.console_label, "用户中心标签", 80)?;
    required(&document.login_label, "登录标签", 80)?;
    required(&document.language_label, "语言标签", 80)?;
    required(&document.utility_navigation_label, "快捷导航标签", 100)?;
    required(&document.github_aria_label, "GitHub 无障碍标签", 180)?;
    required(&document.footer_summary, "页脚摘要", 400)?;
    required(&document.footer_signature, "页脚签名", 180)?;
    required(&document.footer_note, "页脚说明", 400)?;
    validate_links(&document.navigation)?;
    validate_link_item(&document.github)?;
    validate_links(&document.footer_navigation)
}

fn validate_home(document: &EditableHomePage, locale: Locale) -> AppResult<()> {
    if document.schema_version != SITE_CONTENT_SCHEMA_VERSION {
        return Err(AppError::Validation("首页内容 schema_version 无效".into()));
    }
    let baseline = cloud_site::compiled_home_page(locale);
    required(&document.meta_title, "SEO 标题", 120)?;
    required(&document.meta_description, "SEO 描述", 320)?;
    required(&document.eyebrow, "首屏眉题", 120)?;
    required(&document.heading, "首屏标题", 240)?;
    required(&document.lead, "首屏说明", 800)?;
    exact_len(document.actions.len(), baseline.actions.len(), "首页按钮")?;
    validate_links(&document.actions)?;

    let content = &document.content;
    let expected = &baseline.content;
    exact_len(content.platforms.len(), 4, "平台矩阵")?;
    exact_len(
        content.sections.len(),
        expected.sections.len(),
        "首页功能区块",
    )?;
    exact_len(content.faqs.len(), expected.faqs.len(), "首页 FAQ")?;
    required(&content.status_strip_label, "状态条标签", 160)?;
    required(&content.status_note, "状态条说明", 240)?;
    required(&content.hero_blueprint_label, "首屏制图标签", 160)?;
    required(&content.platform_label, "平台标题", 120)?;
    required(&content.platform_note, "平台说明", 80)?;
    required(&content.faq_side_label, "FAQ 侧标", 120)?;
    required(&content.faq_item_prefix, "FAQ 编号前缀", 40)?;
    required(&content.faq_heading, "FAQ 标题", 240)?;
    required(&content.faq_lead, "FAQ 说明", 800)?;
    required(&content.seo_heading, "SEO 主题区标题", 160)?;
    required(&content.seo_topics_label, "SEO 主题区标签", 160)?;
    required(&content.final_heading, "收尾标题", 240)?;
    required(&content.final_lead, "收尾说明", 800)?;
    required(&content.qr_widget.title, "二维码标题", 120)?;
    required(&content.qr_widget.image_alt, "二维码替代文本", 240)?;
    if !matches!(content.media_slot.as_deref(), None | Some("home_qr")) {
        return Err(AppError::Validation("首页媒体槽位只能引用 home_qr".into()));
    }
    for (index, (section, expected_section)) in content
        .sections
        .iter()
        .zip(expected.sections.iter())
        .enumerate()
    {
        if section.anchor != expected_section.anchor || section.layout != expected_section.layout {
            return Err(AppError::Validation(format!(
                "第 {} 个首页区块的结构标识不可编辑",
                index + 1
            )));
        }
        exact_len(
            section.items.len(),
            expected_section.items.len(),
            "首页区块项目",
        )?;
        for (item, expected_item) in section.items.iter().zip(expected_section.items.iter()) {
            if item.tone != expected_item.tone {
                return Err(AppError::Validation("首页卡片视觉类型不可编辑".into()));
            }
        }
    }
    Ok(())
}

fn validate_links(items: &[EditableLink]) -> AppResult<()> {
    for item in items {
        validate_link_item(item)?;
    }
    Ok(())
}

fn validate_link_item(item: &EditableLink) -> AppResult<()> {
    required(&item.label, "链接文字", 120)?;
    link(&item.href)
}

fn required(value: &str, label: &str, max: usize) -> AppResult<()> {
    let length = value.chars().count();
    if value.trim() != value || !(1..=max).contains(&length) {
        return Err(AppError::Validation(format!(
            "{label}必须为 1 到 {max} 个字符且首尾无空白"
        )));
    }
    Ok(())
}

fn exact_len(actual: usize, expected: usize, label: &str) -> AppResult<()> {
    if actual != expected {
        return Err(AppError::Validation(format!(
            "{label}必须保持 {expected} 项结构"
        )));
    }
    Ok(())
}

fn inspect_value(value: &Value, path: &str, depth: usize) -> AppResult<()> {
    if depth > 12 {
        return Err(AppError::Validation("站点内容嵌套层级过深".into()));
    }
    match value {
        Value::String(text) => {
            if text.chars().count() > MAX_TEXT_CHARS
                || text.chars().any(char::is_control)
                || text.contains(['<', '>'])
                || text.contains("{{")
                || text.contains("{%")
            {
                return Err(AppError::Validation(format!("{path} 不是安全的有界纯文本")));
            }
            if path.ends_with("/href") {
                link(text)?;
            }
        }
        Value::Array(items) => {
            if items.len() > 32 {
                return Err(AppError::Validation(format!("{path} 的项目过多")));
            }
            for (index, item) in items.iter().enumerate() {
                inspect_value(item, &format!("{path}/{index}"), depth + 1)?;
            }
        }
        Value::Object(map) => {
            if map.len() > 64 {
                return Err(AppError::Validation(format!("{path} 的字段过多")));
            }
            for (key, item) in map {
                inspect_value(item, &format!("{path}/{key}"), depth + 1)?;
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
    Ok(())
}

fn link(value: &str) -> AppResult<()> {
    if value.starts_with('/') {
        return internal_link(value);
    }
    let parsed = Url::parse(value).map_err(|_| AppError::Validation("外部链接格式无效".into()))?;
    if parsed.scheme() != "https"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || parsed.host_str().is_none()
    {
        return Err(AppError::Validation(
            "外部链接必须是无账号、查询和片段的 HTTPS 地址".into(),
        ));
    }
    Ok(())
}

fn internal_link(value: &str) -> AppResult<()> {
    if value.starts_with("//") || value.contains('?') {
        return Err(AppError::Validation("站内链接格式无效".into()));
    }
    let (path, fragment) = value
        .split_once('#')
        .map_or((value, None), |(path, fragment)| (path, Some(fragment)));
    const ROUTES: &[&str] = &[
        "/",
        "/en",
        "/docs/getting-started",
        "/en/docs/getting-started",
        "/changelog",
        "/en/changelog",
        "/security",
        "/en/security",
        "/downloads",
        "/en/downloads",
        "/faq",
        "/en/faq",
        "/feedback",
        "/en/feedback",
    ];
    if !ROUTES.contains(&path) {
        return Err(AppError::Validation("链接不在受控公开站内路由中".into()));
    }
    if let Some(fragment) = fragment
        && (!path.ends_with("/docs/getting-started")
            || fragment.is_empty()
            || fragment.len() > 64
            || !fragment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
    {
        return Err(AppError::Validation("站内链接片段无效".into()));
    }
    Ok(())
}

fn count_leaves(value: &Value) -> usize {
    match value {
        Value::Array(items) => items.iter().map(count_leaves).sum(),
        Value::Object(map) => map.values().map(count_leaves).sum(),
        _ => 1,
    }
}
