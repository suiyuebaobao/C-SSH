//! 返回当前账号的分页下载历史，匿名事件不会出现在结果中。

use axum::{
    Extension, Json,
    extract::{Query, State},
};
use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};

use crate::{DownloadHistoryItem, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<DownloadHistoryItem>>> {
    Ok(Json(service.account_history(&session, page).await?))
}
