//! 接收浏览器注册表单，创建账号与资料后跳转到用户中心。

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;

use crate::{Register, Service};

use super::form_response;

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(command): Form<Register>,
) -> AppResult<Response> {
    let issued = service.register(command).await?;
    form_response::redirect(
        &headers,
        &issued.raw_token,
        issued.session.expires_at,
        "/console",
    )
}
