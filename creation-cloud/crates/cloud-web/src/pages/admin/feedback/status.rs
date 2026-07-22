//! 处理单条反馈的受控状态迁移表单。
//! 版本校验和状态机由反馈领域再次强制，页面层只解析受限枚举并组织响应。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AuthenticatedSession};
use cloud_feedback::UpdateFeedbackStatusInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::{super::shared, query};

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateStatusForm {
    status: String,
    expected_version: i64,
    reason: String,
    lang: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
    status_filter: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(feedback_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<UpdateStatusForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let status = match query::parse_status(form.status.trim()) {
        Some(status) => status,
        None => {
            return shared::action_error(locale, AppError::Validation("反馈状态值无效".to_owned()));
        }
    };
    let status_filter = match query::optional_status(form.status_filter.as_deref()) {
        Ok(status) => status,
        Err(()) => {
            return shared::action_error(
                locale,
                AppError::Validation("反馈筛选状态值无效".to_owned()),
            );
        }
    };
    match state
        .feedback()
        .update_feedback_status(
            &actor,
            feedback_id,
            UpdateFeedbackStatusInput {
                status,
                expected_version: form.expected_version,
                reason: form.reason,
            },
        )
        .await
    {
        Ok(_) => {
            let path = query::action_return_path(
                feedback_id,
                form.page.unwrap_or(1),
                form.size.unwrap_or(20),
                status_filter,
            );
            shared::action_success(&headers, &path, locale)
        }
        Err(error) => shared::action_error(locale, error),
    }
}
