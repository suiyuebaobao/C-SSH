use axum::http::{HeaderMap, header};

const TRUSTED_CLIENT_IP_HEADER: &str = "x-creation-client-ip";

#[derive(Clone, Debug, Default)]
pub struct TrustedRequestMetadata {
    pub last_login_ip: Option<String>,
    pub user_agent: Option<String>,
}

impl TrustedRequestMetadata {
    #[must_use]
    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            last_login_ip: headers
                .get(TRUSTED_CLIENT_IP_HEADER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.trim().parse::<std::net::IpAddr>().ok())
                .map(|address| address.to_string()),
            user_agent: sanitize_user_agent(
                headers
                    .get(header::USER_AGENT)
                    .and_then(|value| value.to_str().ok()),
            ),
        }
    }
}

fn sanitize_user_agent(value: Option<&str>) -> Option<String> {
    let value = value?;
    let sanitized = value
        .chars()
        .filter(|character| !character.is_control())
        .take(512)
        .collect::<String>();
    let sanitized = sanitized.trim();
    (!sanitized.is_empty()).then(|| sanitized.to_owned())
}
