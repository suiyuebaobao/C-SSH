//! 精确验证公开根地址在非生产与生产环境中的不同安全边界。

use super::public_base_url::resolve;

#[test]
fn development_and_test_allow_default_http_loopback() {
    for environment in ["development", "test"] {
        let parsed = resolve(environment, None).expect("非生产环境应允许默认回环地址");
        assert_eq!(parsed.as_str(), "http://127.0.0.1:8088/");
    }
}

#[test]
fn development_and_test_allow_explicit_http_loopback() {
    let cases = [
        ("development", "http://localhost:3000/"),
        ("test", "http://127.255.255.254:8088"),
        ("test", "http://[::1]:8088/"),
    ];

    for (environment, value) in cases {
        assert!(
            resolve(environment, Some(value)).is_ok(),
            "{environment} 应允许显式 HTTP 回环地址：{value}"
        );
    }
}

#[test]
fn production_requires_an_explicit_public_base_url() {
    for value in [None, Some(""), Some(" \t")] {
        let error = resolve("production", value).expect_err("生产环境缺少有效公开根地址必须失败");
        assert_eq!(
            error.to_string(),
            "production 环境必须显式提供 CLOUD_PUBLIC_BASE_URL"
        );
    }
}

#[test]
fn production_requires_https() {
    let error =
        resolve("production", Some("http://example.com")).expect_err("生产环境 HTTP 地址必须失败");
    assert_eq!(
        error.to_string(),
        "production 环境的 CLOUD_PUBLIC_BASE_URL 必须使用 https"
    );
}

#[test]
fn production_rejects_localhost_names() {
    for value in [
        "https://localhost",
        "https://LOCALHOST./",
        "https://admin.localhost/",
    ] {
        let error = resolve("production", Some(value)).expect_err("生产环境不得使用 localhost");
        assert_eq!(
            error.to_string(),
            "production 环境的 CLOUD_PUBLIC_BASE_URL 禁止使用 localhost 域名",
            "未拒绝 localhost 地址：{value}"
        );
    }
}

#[test]
fn production_rejects_every_ipv4_literal() {
    for value in [
        "https://127.0.0.1",
        "https://127.255.255.254/",
        "https://2130706433/",
        "https://192.0.2.10:8443/",
        "https://198.51.100.20/",
        "https://203.0.113.30/",
    ] {
        let error = resolve("production", Some(value)).expect_err("生产环境不得使用 IPv4 literal");
        assert_eq!(
            error.to_string(),
            "production 环境的 CLOUD_PUBLIC_BASE_URL 必须使用域名，禁止使用 IP literal",
            "未拒绝 IPv4 literal：{value}"
        );
    }
}

#[test]
fn production_rejects_every_ipv6_literal() {
    for value in [
        "https://[::1]/",
        "https://[::ffff:127.0.0.1]/",
        "https://[2001:db8::1]/",
    ] {
        let error = resolve("production", Some(value)).expect_err("生产环境不得使用 IPv6 literal");
        assert_eq!(
            error.to_string(),
            "production 环境的 CLOUD_PUBLIC_BASE_URL 必须使用域名，禁止使用 IP literal",
            "未拒绝 IPv6 literal：{value}"
        );
    }
}

#[test]
fn production_accepts_explicit_public_https_roots() {
    let cases = [
        ("https://example.com", "https://example.com/"),
        ("HTTPS://EXAMPLE.COM/", "https://example.com/"),
        (
            "https://status.example.com:8443",
            "https://status.example.com:8443/",
        ),
        ("https://例子.测试", "https://xn--fsqu00a.xn--0zwm56d/"),
    ];

    for (value, expected) in cases {
        let parsed = resolve("production", Some(value)).expect("公开 HTTPS 根地址应合法");
        assert_eq!(parsed.as_str(), expected);
    }
}

#[test]
fn all_environments_reject_unsafe_or_non_root_urls() {
    let invalid_values = [
        "ftp://example.com",
        "https:///",
        "https://user@example.com",
        "https://:secret@example.com",
        "https://example.com/path",
        "https://example.com/.",
        "https://example.com/%2e/",
        "https://example.com?source=test",
        "https://example.com#section",
    ];

    for value in invalid_values {
        assert!(
            resolve("development", Some(value)).is_err(),
            "不安全地址不应通过校验：{value}"
        );
    }
}
