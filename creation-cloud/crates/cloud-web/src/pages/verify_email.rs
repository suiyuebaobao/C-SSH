//! Renders the bilingual email-verification page.

use askama::Template;
use axum::{extract::Query, response::Html};
use cloud_domain::{AppError, AppResult};
use cloud_site::{Locale, PageId, SiteView, content_service};
use serde::Deserialize;

use crate::seo::SeoHead;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerificationQuery {
    resent: Option<u8>,
}

#[derive(Template)]
#[template(path = "verify-email.html")]
struct VerifyEmailTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
    resent: bool,
}

pub(crate) async fn page(Query(query): Query<VerificationQuery>) -> AppResult<Html<String>> {
    render(Locale::ZhCn, query.resent == Some(1))
}

pub(crate) async fn page_en(Query(query): Query<VerificationQuery>) -> AppResult<Html<String>> {
    render(Locale::En, query.resent == Some(1))
}

fn render(locale: Locale, resent: bool) -> AppResult<Html<String>> {
    let is_en = locale == Locale::En;
    let mut view = content_service().view(PageId::Register, locale);
    view.page.meta_title = if is_en {
        "Verify email | Creation Cloud"
    } else {
        "验证邮箱｜Creation Cloud"
    }
    .to_owned();
    view.page.meta_description = if is_en {
        "Verify the email address for a Creation Cloud account."
    } else {
        "验证 Creation Cloud 账号的邮箱地址。"
    }
    .to_owned();
    view.shell.language_href = if is_en {
        "/verify-email"
    } else {
        "/en/verify-email"
    }
    .to_owned();
    VerifyEmailTemplate {
        view,
        seo: SeoHead::private(),
        is_en,
        resent,
    }
    .render()
    .map(Html)
    .map_err(|_| AppError::Internal("email verification page cannot be rendered".to_owned()))
}
