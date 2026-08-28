//! 接收受控发布工具的四资产安装身份 JSON，不向后台 HTML 暴露输入框。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{RecordInstalledIdentitiesInput, RecordInstalledIdentitiesResult, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<RecordInstalledIdentitiesInput>,
) -> AppResult<Json<RecordInstalledIdentitiesResult>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .record_release_installed_identities(&actor, input)
        .await
        .map(Json)
}
