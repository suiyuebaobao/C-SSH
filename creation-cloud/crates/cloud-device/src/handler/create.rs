//! 绑定设备、轮换长期会话 Cookie 并返回设备与新会话视图。

use axum::{
    Extension, Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::{CreateDevice, DeviceSessionResult, Service, session};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(current_session): Extension<AuthenticatedSession>,
    Json(command): Json<CreateDevice>,
) -> AppResult<Response> {
    let outcome = service.create(&current_session, command).await?;
    let status = if outcome.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    let cookie = session::cookie(&outcome.raw_token, outcome.session.idle_expires_at)?;
    let body = DeviceSessionResult {
        device: outcome.device,
        session: outcome.session,
    };
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(header::SET_COOKIE, cookie);
    Ok(response)
}
