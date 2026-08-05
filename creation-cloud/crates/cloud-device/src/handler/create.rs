//! 绑定设备、轮换长期会话 Cookie 并返回设备与新会话视图。

use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppResult, AuthenticatedSession};

use crate::session;
use crate::{
    CreateDevice, DeviceSessionResult, Service, use_case::create::TrustedRequestMetadata,
    validation,
};

const TRUSTED_CLIENT_IP_HEADER: &str = "x-creation-client-ip";

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(current_session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Json(command): Json<CreateDevice>,
) -> AppResult<Response> {
    let metadata = TrustedRequestMetadata {
        last_login_ip: validation::trusted_ip(
            headers
                .get(TRUSTED_CLIENT_IP_HEADER)
                .and_then(|value| value.to_str().ok()),
        ),
        user_agent: validation::user_agent(
            headers
                .get(header::USER_AGENT)
                .and_then(|value| value.to_str().ok()),
        ),
    };
    let outcome = service
        .create_with_metadata(&current_session, command, metadata)
        .await?;
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
