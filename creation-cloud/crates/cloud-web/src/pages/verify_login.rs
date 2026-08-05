//! 渲染普通用户双语登录邮箱验证码页面。

use askama::Template;
use axum::{extract::Query, response::Html};
use cloud_domain::{AppError, AppResult};
use cloud_site::{Locale, PageId, SiteView, content_service};
use serde::Deserialize;
use url::form_urlencoded::Serializer;
use uuid::Uuid;

use crate::{pages::account::safe_next, seo::SeoHead};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoginVerificationQuery {
    challenge_id: Uuid,
    next: Option<String>,
    resent: Option<u8>,
    lang: Option<String>,
}

#[derive(Template)]
#[template(path = "verify-login.html")]
struct VerifyLoginTemplate {
    view: SiteView,
    seo: SeoHead,
    is_en: bool,
    challenge_id: Uuid,
    next_path: Option<String>,
    resent: bool,
}

pub(crate) async fn page(Query(query): Query<LoginVerificationQuery>) -> AppResult<Html<String>> {
    let locale = if query.lang.as_deref() == Some("en") {
        Locale::En
    } else {
        Locale::ZhCn
    };
    render(locale, query)
}

pub(crate) async fn page_en(
    Query(query): Query<LoginVerificationQuery>,
) -> AppResult<Html<String>> {
    render(Locale::En, query)
}

fn render(locale: Locale, query: LoginVerificationQuery) -> AppResult<Html<String>> {
    let is_en = locale == Locale::En;
    let next_path = safe_next(query.next);
    let mut view = content_service().view(PageId::Login, locale);
    view.page.meta_title = if is_en {
        "Verify sign-in | Creation Cloud"
    } else {
        "验证登录｜Creation Cloud"
    }
    .to_owned();
    view.page.meta_description = if is_en {
        "Verify a Creation Cloud sign-in with the code sent to the account email."
    } else {
        "使用账号邮箱收到的验证码确认 Creation Cloud 登录。"
    }
    .to_owned();
    view.shell.language_href = verification_href(
        if is_en {
            "/verify-login"
        } else {
            "/en/verify-login"
        },
        query.challenge_id,
        next_path.as_deref(),
        query.resent == Some(1),
    );
    VerifyLoginTemplate {
        view,
        seo: SeoHead::private(),
        is_en,
        challenge_id: query.challenge_id,
        next_path,
        resent: query.resent == Some(1),
    }
    .render()
    .map(Html)
    .map_err(|_| AppError::Internal("login verification page cannot be rendered".to_owned()))
}

fn verification_href(
    path: &str,
    challenge_id: Uuid,
    next_path: Option<&str>,
    resent: bool,
) -> String {
    let mut query = Serializer::new(String::new());
    query.append_pair("challenge_id", &challenge_id.to_string());
    if let Some(next_path) = next_path {
        query.append_pair("next", next_path);
    }
    if resent {
        query.append_pair("resent", "1");
    }
    format!("{path}?{}", query.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_link_preserves_only_safe_local_state() {
        let challenge_id = Uuid::now_v7();
        let href = verification_href(
            "/en/verify-login",
            challenge_id,
            Some("/admin/releases?lang=en"),
            true,
        );
        assert!(href.starts_with("/en/verify-login?challenge_id="));
        assert!(href.contains("next=%2Fadmin%2Freleases%3Flang%3Den"));
        assert!(href.ends_with("resent=1"));
    }
}
