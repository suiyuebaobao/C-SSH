//! 组装登录账号通知与管理员定向投递路由，权限链由服务端挂载层提供。

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{
    AdminNotification, CreateNotificationInput, NotificationListQuery, NotificationListResponse,
    NotificationReceipt, ReceiptInput, Service,
};

#[must_use = "账号通知路由必须挂载到已认证与 CSRF 保护的 API"]
pub fn account_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_account))
        .route("/{notification_id}/receipt", axum::routing::post(receipt))
        .with_state(service)
}

#[must_use = "通知管理路由必须挂载到管理员认证链"]
pub fn management_router(service: Service) -> Router {
    Router::new()
        .route("/", get(list_admin).post(create_admin))
        .with_state(service)
}

async fn list_account(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<NotificationListQuery>,
) -> AppResult<Json<NotificationListResponse>> {
    service
        .list_account(session.account_id, query)
        .await
        .map(Json)
}

async fn receipt(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(notification_id): Path<Uuid>,
    Json(input): Json<ReceiptInput>,
) -> AppResult<Json<NotificationReceipt>> {
    service
        .receipt(session.account_id, notification_id, input)
        .await
        .map(Json)
}

async fn list_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<AdminNotification>>> {
    let actor = AdminActor::from_session(&session)?;
    service.list_admin(&actor, page).await.map(Json)
}

async fn create_admin(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<CreateNotificationInput>,
) -> AppResult<(StatusCode, Json<AdminNotification>)> {
    let actor = AdminActor::from_session(&session)?;
    service
        .create_admin(&actor, input)
        .await
        .map(|record| (StatusCode::CREATED, Json(record)))
}
