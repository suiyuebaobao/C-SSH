//! Sends a browser registration to the verification page without issuing a session.

use axum::{
    Form,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};

use crate::{Register, Service};

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(command): Form<Register>,
) -> AppResult<Response> {
    let destination = if command.locale == "en" {
        "/en/verify-email"
    } else {
        "/verify-email"
    };
    service.register(command).await?;
    let htmx = headers
        .get("hx-request")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    let mut response = if htmx {
        StatusCode::OK.into_response()
    } else {
        StatusCode::SEE_OTHER.into_response()
    };
    let name = if htmx {
        HeaderName::from_static("hx-redirect")
    } else {
        header::LOCATION
    };
    response.headers_mut().insert(
        name,
        HeaderValue::from_str(destination)
            .map_err(|_| AppError::Internal("registration redirect is invalid".to_owned()))?,
    );
    Ok(response)
}
