//! 接收浏览器登录表单，建立会话后跳转到用户中心。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;

use crate::{Login, Service};

use super::form_response;

#[derive(Deserialize)]
pub(crate) struct BrowserLogin {
    email: String,
    password: String,
    next: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserLogin>,
) -> AppResult<Response> {
    let destination = form_response::safe_destination(form.next.as_deref());
    let command = Login {
        email: form.email,
        password: form.password,
    };
    let issued = service.login(command).await?;
    form_response::redirect(
        &headers,
        &issued.raw_token,
        issued.session.expires_at,
        destination,
    )
}
