//! 展示首页二维码发布槽位的当前版本与受控历史。
//! 上传、替代文本更新、发布、撤销和删除分别由独立处理器承担。

pub(crate) mod content;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod publish;
pub(crate) mod revoke;
pub(crate) mod update;

use askama::Template;
use axum::{
    Extension,
    extract::{Query, State},
    response::Html,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_site::{Locale, PageId, SiteView};
use cloud_site_content::{
    SiteContentDocumentKey, SiteContentListQuery, SiteContentRevision, SiteContentState,
};
use cloud_site_media::{PublicHomeQr, SiteMedia};
use serde::Deserialize;

use crate::{AdminPageState, seo::SeoHead};

use super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SiteQuery {
    lang: Option<String>,
    section: Option<String>,
    content_lang: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SiteSection {
    Home,
    SiteShell,
    Qr,
}

impl SiteSection {
    fn parse(value: Option<&str>) -> AppResult<Self> {
        match value.map(str::trim) {
            None | Some("") | Some("home") => Ok(Self::Home),
            Some("site_shell") => Ok(Self::SiteShell),
            Some("qr") => Ok(Self::Qr),
            Some(_) => Err(AppError::Validation("网站内容分区无效".to_owned())),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::SiteShell => "site_shell",
            Self::Qr => "qr",
        }
    }

    const fn document_key(self) -> Option<SiteContentDocumentKey> {
        match self {
            Self::Home => Some(SiteContentDocumentKey::Home),
            Self::SiteShell => Some(SiteContentDocumentKey::SiteShell),
            Self::Qr => None,
        }
    }
}

struct CurrentMedia {
    content_url: String,
    alt_zh: String,
    alt_en: String,
    dimensions: String,
    published_at: String,
}

struct MediaRow {
    id: String,
    state: &'static str,
    content_type: String,
    byte_size: i64,
    sha256: String,
    dimensions: String,
    alt_zh: String,
    alt_en: String,
    created_at: String,
}

struct ContentHistoryRow {
    id: String,
    document_label_zh: &'static str,
    document_label_en: &'static str,
    locale_label_zh: &'static str,
    locale_label_en: &'static str,
    state: &'static str,
    revision: i64,
    updated_at: String,
    is_published: bool,
}

#[derive(Template)]
#[template(path = "admin-site.html")]
struct SiteTemplate {
    view: SiteView,
    seo: SeoHead,
    session_identity: Option<String>,
    csrf_token: String,
    is_en: bool,
    section: &'static str,
    document_key: &'static str,
    content_lang: &'static str,
    current: Option<CurrentMedia>,
    rows: Vec<MediaRow>,
    load_error: Option<String>,
    content_editors: Vec<content::ContentEditor>,
    content_history: Vec<ContentHistoryRow>,
    content_error: Option<String>,
}

pub(crate) async fn page(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<SiteQuery>,
) -> AppResult<Html<String>> {
    let locale = shared::locale(query.lang.as_deref());
    let section = SiteSection::parse(query.section.as_deref())?;
    let content_locale = selected_content_locale(query.content_lang.as_deref(), locale)?;
    let actor = shared::actor_from_session(&session)?;
    let (content_editors, content_history, content_error) =
        load_content(&state, &actor, section, content_locale, locale).await;
    let (current, rows, load_error) = load_media(&state, &actor, section, locale).await;
    let parts = shared::page_parts(PageId::AdminSite, locale, &session);
    shared::render(&SiteTemplate {
        view: parts.view,
        seo: parts.seo,
        session_identity: Some(parts.session_identity),
        csrf_token: parts.csrf_token,
        is_en: parts.is_en,
        section: section.as_str(),
        document_key: section.document_key().map_or("", |value| value.as_str()),
        content_lang: content_locale.code(),
        current,
        rows,
        load_error,
        content_editors,
        content_history,
        content_error,
    })
}

async fn load_content(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
    section: SiteSection,
    content_locale: Locale,
    ui_locale: Locale,
) -> (
    Vec<content::ContentEditor>,
    Vec<ContentHistoryRow>,
    Option<String>,
) {
    let Some(document_key) = section.document_key() else {
        return (Vec::new(), Vec::new(), None);
    };
    match state
        .site_content()
        .list(
            actor,
            SiteContentListQuery {
                document_key: Some(document_key),
                locale: Some(content_locale),
            },
        )
        .await
    {
        Ok(records) => match split_content(records) {
            Ok((editors, history)) => (editors, history, None),
            Err(_) => (Vec::new(), Vec::new(), Some(content_load_error(ui_locale))),
        },
        Err(_) => (Vec::new(), Vec::new(), Some(content_load_error(ui_locale))),
    }
}

async fn load_media(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
    section: SiteSection,
    locale: Locale,
) -> (Option<CurrentMedia>, Vec<MediaRow>, Option<String>) {
    if section != SiteSection::Qr {
        return (None, Vec::new(), None);
    }
    let (rows, list_error) = match state.site_media().list(actor, Some(100)).await {
        Ok(items) => (items.into_iter().map(MediaRow::from).collect(), None),
        Err(_) => (Vec::new(), Some(media_load_error(locale))),
    };
    let (current, current_error) = match state.site_media().current_home_qr().await {
        Ok(media) => (Some(CurrentMedia::from(media)), None),
        Err(AppError::NotFound(_)) => (None, None),
        Err(_) => (None, Some(current_media_load_error(locale))),
    };
    (current, rows, list_error.or(current_error))
}

fn selected_content_locale(value: Option<&str>, ui_locale: Locale) -> AppResult<Locale> {
    match value.map(str::trim) {
        None | Some("") => Ok(ui_locale),
        Some("zh-CN") => Ok(Locale::ZhCn),
        Some("en") => Ok(Locale::En),
        Some(_) => Err(AppError::Validation("网站内容语种无效".to_owned())),
    }
}

pub(crate) fn content_return_path(document_key: SiteContentDocumentKey, locale: Locale) -> String {
    let section = match document_key {
        SiteContentDocumentKey::Home => SiteSection::Home,
        SiteContentDocumentKey::SiteShell => SiteSection::SiteShell,
    };
    format!(
        "/admin/site?section={}&content_lang={}",
        section.as_str(),
        locale.code()
    )
}

pub(crate) const fn qr_return_path() -> &'static str {
    "/admin/site?section=qr"
}

fn split_content(
    records: Vec<SiteContentRevision>,
) -> AppResult<(Vec<content::ContentEditor>, Vec<ContentHistoryRow>)> {
    let mut editors = Vec::new();
    let mut history = Vec::new();
    for record in records {
        if record.state == SiteContentState::Draft {
            editors.push(content::ContentEditor::try_from(record)?);
        } else {
            history.push(ContentHistoryRow::from(record));
        }
    }
    Ok((editors, history))
}

fn content_load_error(locale: Locale) -> String {
    if locale == Locale::En {
        "Structured site content is temporarily unavailable.".to_owned()
    } else {
        "结构化站点内容暂时无法读取。".to_owned()
    }
}

fn media_load_error(locale: Locale) -> String {
    if locale == Locale::En {
        "Site media history is temporarily unavailable.".to_owned()
    } else {
        "站点媒体历史暂时无法读取。".to_owned()
    }
}

fn current_media_load_error(locale: Locale) -> String {
    if locale == Locale::En {
        "The current publication state is temporarily unavailable.".to_owned()
    } else {
        "当前发布状态暂时无法读取。".to_owned()
    }
}

impl From<PublicHomeQr> for CurrentMedia {
    fn from(value: PublicHomeQr) -> Self {
        Self {
            content_url: value.content_url,
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            dimensions: format!("{}×{}", value.width, value.height),
            published_at: value.published_at.to_rfc3339(),
        }
    }
}

impl From<SiteMedia> for MediaRow {
    fn from(value: SiteMedia) -> Self {
        Self {
            id: value.id.to_string(),
            state: value.state.as_str(),
            content_type: value.content_type,
            byte_size: value.byte_size,
            sha256: value.sha256,
            dimensions: format!("{}×{}", value.width, value.height),
            alt_zh: value.alt_zh,
            alt_en: value.alt_en,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<SiteContentRevision> for ContentHistoryRow {
    fn from(value: SiteContentRevision) -> Self {
        Self {
            id: value.id.to_string(),
            document_label_zh: match value.document_key {
                cloud_site_content::SiteContentDocumentKey::SiteShell => "公共页头与页脚",
                cloud_site_content::SiteContentDocumentKey::Home => "首页正文",
            },
            document_label_en: match value.document_key {
                cloud_site_content::SiteContentDocumentKey::SiteShell => "Shared header and footer",
                cloud_site_content::SiteContentDocumentKey::Home => "Home page",
            },
            locale_label_zh: match value.locale {
                Locale::ZhCn => "简体中文",
                Locale::En => "英文",
            },
            locale_label_en: match value.locale {
                Locale::ZhCn => "Chinese",
                Locale::En => "English",
            },
            state: value.state.as_str(),
            revision: value.revision,
            updated_at: value.updated_at.to_rfc3339(),
            is_published: value.state == SiteContentState::Published,
        }
    }
}
