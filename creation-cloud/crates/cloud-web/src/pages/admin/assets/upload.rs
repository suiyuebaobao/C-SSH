//! 将安装包 multipart 流直接交给下载领域的流式上传用例。
//! 页面层绝不把安装包读入内存，也不自行生成落盘路径或绕过 SHA256 核验。

use axum::{
    Extension,
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::AuthenticatedSession;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct UploadQuery {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(asset_id): Path<Uuid>,
    Query(query): Query<UploadQuery>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let locale = shared::locale(query.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    match state
        .download()
        .upload_asset(&actor, asset_id, multipart)
        .await
    {
        Ok(_) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
