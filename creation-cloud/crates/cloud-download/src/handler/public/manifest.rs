//! 返回只包含已发布版本和已启用来源的公开清单。

use axum::{Json, extract::State};
use cloud_domain::AppResult;

use crate::{PublicRelease, Service};

pub(crate) async fn handle(State(service): State<Service>) -> AppResult<Json<Vec<PublicRelease>>> {
    Ok(Json(service.public_manifest().await?))
}
