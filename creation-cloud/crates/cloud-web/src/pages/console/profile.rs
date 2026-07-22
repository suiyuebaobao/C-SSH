//! 渲染当前账号资料与安全页面，并把资料修改交给独立动作。

pub(crate) mod change_password;
pub(crate) mod update;

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};
use cloud_user::Profile;

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

#[derive(Template)]
#[template(path = "console-profile.html")]
struct ProfileTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    email: String,
    role: String,
    profile: Profile,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let profile = state.user().get(&session, session.account_id).await?;
    let locale = query.locale();
    common::render(&ProfileTemplate {
        view: common::view(PageId::Profile, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        email: session.email,
        role: session.role,
        profile,
    })
}
