//! 保存逐设备主机下载白名单。

use axum::{
    Extension,
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_host::ReplaceAllowlistRequest;
use cloud_site::Locale;
use uuid::Uuid;

use crate::ConsolePageState;

use super::super::common;

pub(crate) async fn handle(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(device_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Response> {
    let mut host_ids = Vec::new();
    let mut locale = Locale::ZhCn;
    for (name, value) in url::form_urlencoded::parse(&body) {
        match name.as_ref() {
            "host_id" => match value.parse::<Uuid>() {
                Ok(id) if !id.is_nil() => host_ids.push(id),
                _ => {
                    return Err(AppError::Validation("主机标识无效".to_owned()));
                }
            },
            "lang" if value == "en" => locale = Locale::En,
            _ => {}
        }
    }
    host_ids.sort_unstable();
    host_ids.dedup();
    state
        .host()
        .replace_download_allowlist(&session, device_id, ReplaceAllowlistRequest { host_ids })
        .await?;
    Ok(common::action_success_to(
        &headers,
        "/console/hosts",
        locale,
    ))
}
