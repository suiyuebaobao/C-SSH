//! 提供只含 typed code 与匿名 UUID 的管理员定向账号通知表单。

use axum::{Extension, Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::{AppError, AuthenticatedSession};
use cloud_notification::{CreateNotificationInput, NotificationKind, NotificationPriority};
use serde::Deserialize;
use uuid::Uuid;

use crate::AdminPageState;

use super::shared;

#[derive(Deserialize)]
pub(crate) struct NotificationForm {
    account_id: String,
    code: String,
    priority: String,
    resource_id: Option<String>,
    lang: Option<String>,
}

pub(crate) async fn create(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<NotificationForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = match parse(form) {
        Ok(value) => value,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.notification().create_admin(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/announcements", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

fn parse(form: NotificationForm) -> Result<CreateNotificationInput, AppError> {
    let account_id = parse_uuid(&form.account_id, "目标账号")?;
    let resource_id = form
        .resource_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_uuid(value, "匿名资源"))
        .transpose()?;
    let kind = if form.code.starts_with("sync_") {
        NotificationKind::Sync
    } else {
        NotificationKind::AccountSecurity
    };
    let priority = match form.priority.as_str() {
        "normal" => NotificationPriority::Normal,
        "important" => NotificationPriority::Important,
        "critical" => NotificationPriority::Critical,
        _ => return Err(AppError::Validation("通知优先级无效".into())),
    };
    Ok(CreateNotificationInput {
        account_id,
        kind,
        priority,
        code: form.code,
        resource_id,
        expires_at: None,
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, AppError> {
    value
        .trim()
        .parse::<Uuid>()
        .map_err(|_| AppError::Validation(format!("{field}标识无效")))
}

#[cfg(test)]
mod tests {
    const TEMPLATE: &str = include_str!("../../../templates/admin-announcements.html");

    #[test]
    fn targeted_notification_form_has_no_free_title_body_or_secret_parameter() {
        for field in ["name=\"account_id\"", "name=\"code\"", "name=\"priority\""] {
            assert!(TEMPLATE.contains(field), "定向通知后台缺少字段 {field}");
        }
        assert!(!TEMPLATE.contains("name=\"notification_title\""));
        assert!(!TEMPLATE.contains("name=\"notification_body\""));
        assert!(!TEMPLATE.contains("name=\"parameters\""));
    }
}
