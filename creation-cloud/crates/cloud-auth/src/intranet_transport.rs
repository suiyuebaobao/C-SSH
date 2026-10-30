//! 显式内网HTTP的认证Cookie隔离适配；不依赖请求头来降低HTTPS策略。

use axum::{
    extract::Request,
    http::{
        HeaderMap, HeaderValue,
        header::{COOKIE, SET_COOKIE},
    },
    middleware::Next,
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult};

pub async fn cookies(mut request: Request, next: Next) -> Response {
    let mut response = match adapt_request(request.headers_mut()) {
        Ok(()) => next.run(request).await,
        Err(error) => error.into_response(),
    };
    if let Err(error) = adapt_response(response.headers_mut()) {
        response = error.into_response();
    }
    response.headers_mut().insert(
        "x-cssh-cloud-transport",
        HeaderValue::from_static("intranet-http-v1"),
    );
    response
}

fn http_name(name: &str) -> String {
    format!(
        "creation_intranet_{}",
        name.strip_prefix("creation_").unwrap_or(name)
    )
}

fn header(value: &str) -> AppResult<HeaderValue> {
    let mut value = HeaderValue::from_str(value)
        .map_err(|_| AppError::Validation("认证Cookie格式无效".to_owned()))?;
    value.set_sensitive(true);
    Ok(value)
}

fn adapt_request(headers: &mut HeaderMap) -> AppResult<()> {
    let names = super::cookie::transport_cookie_names();
    let mut retained = Vec::new();
    for value in headers.get_all(COOKIE) {
        let raw = value
            .to_str()
            .map_err(|_| AppError::Validation("认证Cookie格式无效".to_owned()))?;
        for pair in raw.split(';') {
            let Some((name, value)) = pair.trim().split_once('=') else {
                continue;
            };
            if names.contains(&name) {
                continue;
            }
            if let Some(original) = names.iter().find(|original| http_name(original) == name) {
                retained.push(format!("{original}={value}"));
            } else {
                retained.push(pair.trim().to_owned());
            }
        }
    }
    headers.remove(COOKIE);
    if !retained.is_empty() {
        headers.insert(COOKIE, header(&retained.join("; "))?);
    }
    Ok(())
}

fn adapt_response(headers: &mut HeaderMap) -> AppResult<()> {
    let names = super::cookie::transport_cookie_names();
    let mut updated = Vec::new();
    for value in headers.get_all(SET_COOKIE) {
        let raw = value
            .to_str()
            .map_err(|_| AppError::Internal("认证Cookie响应无效".to_owned()))?;
        let (pair, tail) = raw.split_once(';').unwrap_or((raw, ""));
        let Some((name, value)) = pair.split_once('=') else {
            updated.push(header(raw)?);
            continue;
        };
        if !names.contains(&name) {
            updated.push(header(raw)?);
            continue;
        }
        let attributes = tail
            .split(';')
            .map(str::trim)
            .filter(|attribute| !attribute.is_empty() && !attribute.eq_ignore_ascii_case("secure"))
            .collect::<Vec<_>>();
        let mut output = format!("{}={value}", http_name(name));
        if !attributes.is_empty() {
            output.push_str("; ");
            output.push_str(&attributes.join("; "));
        }
        updated.push(header(&output)?);
    }
    headers.remove(SET_COOKIE);
    for value in updated {
        headers.append(SET_COOKIE, value);
    }
    Ok(())
}

