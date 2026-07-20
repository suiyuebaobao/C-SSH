//! 处理单条反馈标题与正文的不可逆安全脱敏表单。
//! 页面层要求独立确认，领域层仍以版本锁和数据库事务保证不可逆语义。

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AuthenticatedSession};
use cloud_feedback::RedactFeedbackInput;
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::{super::shared, query};

#[derive(Debug, Deserialize)]
pub(crate) struct RedactForm {
    expected_version: i64,
    reason: String,
    confirmed: Option<String>,
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
    Form(form): Form<RedactForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    if form.confirmed.as_deref() != Some("redact") {
        return shared::action_error(
            locale,
            AppError::Validation("必须显式确认不可逆安全脱敏".to_owned()),
        );
    }
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
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
        .redact_feedback(
            &actor,
            feedback_id,
            RedactFeedbackInput {
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
