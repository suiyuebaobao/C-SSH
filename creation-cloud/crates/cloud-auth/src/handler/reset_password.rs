//! Confirms a password-reset challenge without issuing a new session.

use axum::{Json, extract::State, http::StatusCode};
use cloud_domain::AppResult;

use crate::{ResetPassword, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Json(command): Json<ResetPassword>,
) -> AppResult<StatusCode> {
    let _ = service.reset_password(command).await?;
    Ok(StatusCode::NO_CONTENT)
}
