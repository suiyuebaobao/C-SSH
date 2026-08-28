//! 暴露管理员版本策略草稿、二次确认发布与只读快照端点。

use axum::{Extension, Json, extract::State};
use cloud_domain::{AdminActor, AppResult, AuthenticatedSession};

use crate::{
    AdminUpdatePolicySnapshot, PublishUpdatePolicyInput, PublishedUpdatePolicy,
    SaveUpdatePolicyDraftInput, Service, UpdatePolicyDraft,
};

pub(crate) async fn get(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
) -> AppResult<Json<AdminUpdatePolicySnapshot>> {
    let actor = AdminActor::from_session(&session)?;
    service.admin_update_policy(&actor).await.map(Json)
}

pub(crate) async fn save_draft(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<SaveUpdatePolicyDraftInput>,
) -> AppResult<Json<UpdatePolicyDraft>> {
    let actor = AdminActor::from_session(&session)?;
    service
        .save_update_policy_draft(&actor, input)
        .await
        .map(Json)
}

pub(crate) async fn publish(
    State(service): State<Service>,
    Extension(session): Extension<AuthenticatedSession>,
    Json(input): Json<PublishUpdatePolicyInput>,
) -> AppResult<Json<PublishedUpdatePolicy>> {
    let actor = AdminActor::from_session(&session)?;
    service.publish_update_policy(&actor, input).await.map(Json)
}
