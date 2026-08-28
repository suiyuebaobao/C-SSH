//! 仅接收一个小型 signature 文件，禁止管理 API 直接提交任意签名字符串。

use axum::{
    Extension,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, signature::MAX_SIGNATURE_BYTES};

pub(crate) async fn upload(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<StatusCode> {
    let actor = AdminActor::from_session(&session)?;
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
    let signature = signature.ok_or_else(|| AppError::Validation("缺少 signature 文件".into()))?;
    service
        .set_asset_updater_signature(&actor, asset_id, &signature)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
