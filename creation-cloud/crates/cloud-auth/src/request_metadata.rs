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

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::TrustedRequestMetadata;

    #[test]
    fn metadata_accepts_only_a_single_valid_trusted_ip_and_sanitizes_user_agent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-creation-client-ip",
            HeaderValue::from_static("2001:db8::1"),
        );
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_bytes(b"Creation\tClient 7.0").expect("valid header"),
        );
        let metadata = TrustedRequestMetadata::from_headers(&headers);
        assert_eq!(metadata.last_login_ip.as_deref(), Some("2001:db8::1"));
        assert_eq!(metadata.user_agent.as_deref(), Some("CreationClient 7.0"));

        headers.insert(
            "x-creation-client-ip",
            HeaderValue::from_static("198.51.100.1, 203.0.113.2"),
        );
        assert!(
            TrustedRequestMetadata::from_headers(&headers)
                .last_login_ip
                .is_none()
        );
    }
}
