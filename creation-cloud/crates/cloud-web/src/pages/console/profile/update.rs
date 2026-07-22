//! 接收资料表单并仅更新当前账号拥有的资料。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::PageId;
use cloud_user::UpdateProfile;
use serde::Deserialize;

use crate::ConsolePageState;

use super::super::common;

#[derive(Deserialize)]
pub(crate) struct UpdateProfileForm {
    display_name: String,
    locale: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<UpdateProfileForm>,
) -> AppResult<Response> {
    let locale = common::locale(form.lang.as_deref());
    state
        .user()
        .update(
            &session,
            session.account_id,
            UpdateProfile {
                display_name: Some(form.display_name),
                locale: Some(form.locale),
            },
        )
        .await?;
    Ok(common::action_success(&headers, PageId::Profile, locale))
}
