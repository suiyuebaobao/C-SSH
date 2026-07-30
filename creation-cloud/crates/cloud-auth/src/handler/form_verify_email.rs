//! Handles the browser email-verification form without putting the code in a URL.

use axum::{Form, extract::State, http::HeaderMap, response::Response};
use cloud_domain::AppResult;
use serde::Deserialize;

use crate::{Service, VerifyEmail};

use super::form_response;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BrowserVerifyEmail {
    email: String,
    code: String,
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(service): State<Service>,
    headers: HeaderMap,
    Form(form): Form<BrowserVerifyEmail>,
) -> AppResult<Response> {
    let destination = if form.lang.as_deref() == Some("en") {
        "/console?lang=en"
    } else {
        "/console"
    };
    let issued = service
        .verify_email(VerifyEmail {
            email: form.email,
            code: form.code,
        })
        .await?;
    form_response::redirect(
        &headers,
        &issued.raw_token,
        issued.metadata.idle_expires_at,
        destination,
    )
}
