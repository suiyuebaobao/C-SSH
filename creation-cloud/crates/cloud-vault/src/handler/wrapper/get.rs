//! 仅从 AuthenticatedSession 获取身份并映射单条包装密钥读取请求。

use axum::{Extension, Json, extract::Path, extract::State};
use cloud_domain::{AppResult, AuthenticatedSession};
use uuid::Uuid;

use crate::{Service, VaultKeyWrapper};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(wrapper_id): Path<Uuid>,
) -> AppResult<Json<VaultKeyWrapper>> {
    service.get_wrapper(&session, wrapper_id).await.map(Json)
}
