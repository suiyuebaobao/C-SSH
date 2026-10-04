//! 提供前后端分离所需的最小认证策略和管理员设置JSON适配，复用认证领域用例。

use crate::{AuthSettings, Service, UpdateAuthSettings};
use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct PublicPolicy {
    user_captcha_enabled: bool,
    admin_captcha_enabled: bool,
}

pub(crate) async fn policy(State(service): State<Service>) -> AppResult<Json<PublicPolicy>> {
    let settings = service.login_captcha_settings().await?;
    Ok(Json(PublicPolicy {
        user_captcha_enabled: settings.user_captcha_enabled,
        admin_captcha_enabled: settings.admin_captcha_enabled,
    }))
}

pub(crate) async fn get(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<AuthSettings>> {
    Ok(Json(
        service
            .auth_settings(&AdminActor::from_session(&session)?)
            .await?,
    ))
}

pub(crate) async fn update(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<UpdateAuthSettings>,
) -> AppResult<Json<AuthSettings>> {
    Ok(Json(
        service
            .update_auth_settings(&AdminActor::from_session(&session)?, input)
            .await?,
    ))
}
