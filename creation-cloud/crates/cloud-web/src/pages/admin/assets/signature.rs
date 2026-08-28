//! 将后台选择的小型 signature 文件交给下载领域，不在页面层解释签名内容。

use axum::{
    Extension,
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

const MAX_SIGNATURE_BYTES: usize = 8192;

#[derive(Default, Deserialize)]
pub(crate) struct SignatureQuery {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    Query(query): Query<SignatureQuery>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let locale = shared::locale(query.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let signature = match read_signature(multipart).await {
        Ok(value) => value,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .download()
        .set_asset_updater_signature(&actor, asset_id, &signature)
        .await
    {
        Ok(()) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

async fn read_signature(mut multipart: Multipart) -> AppResult<String> {
    let mut signature = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("签名上传表单无效".into()))?
    {
        if field.name() != Some("signature") || signature.is_some() {
            return Err(AppError::Validation(
                "签名上传只允许一个 signature 文件".into(),
            ));
        }
        if field.file_name().is_none_or(|name| name.trim().is_empty()) {
            return Err(AppError::Validation("请选择 signature 文件".into()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| AppError::Validation("读取 signature 文件失败".into()))?
        {
            if bytes.len().saturating_add(chunk.len()) > MAX_SIGNATURE_BYTES {
                return Err(AppError::Validation("signature 文件超过 8192 字节".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        signature = Some(
            String::from_utf8(bytes)
                .map_err(|_| AppError::Validation("signature 文件必须是 UTF-8".into()))?,
        );
    }
    signature.ok_or_else(|| AppError::Validation("缺少 signature 文件".into()))
}
