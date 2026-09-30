//! 验证公开路由与需要管理员认证的来源写路由彼此隔离。

use std::path::PathBuf;

use axum::{
    body::{Body, to_bytes},
    extract::Extension,
    http::{Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{Service, management_router, public_router, update_router};

fn service() -> Service {
    let Ok(pool): Result<PgPool, _> = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://example:example@example.com/example")
    else {
        panic!("固定测试连接串应可创建惰性连接池");
    };
    Service::new(pool, PathBuf::from("releases"))
}

fn user_session() -> AuthenticatedSession {
    AuthenticatedSession {
        account_id: Uuid::now_v7(),
        email: "user@example.com".into(),
        admin_login_name: None,
        role: "user".into(),
        device_id: None,
        expires_at: Utc::now() + Duration::minutes(5),
        csrf_token: "csrf-example".into(),
        session_id: Uuid::now_v7(),
    }
}

#[tokio::test]
async fn public_router_rejects_source_writes() {
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/assets/{}/sources", Uuid::now_v7()))
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = public_router(service())
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn management_router_has_no_public_manifest() {
    let request = Request::builder()
        .uri("/releases")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = management_router(service())
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn management_handler_rejects_non_admin_before_database_access() {
    let request = Request::builder()
        .uri(format!("/assets/{}/sources", Uuid::now_v7()))
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = management_router(service())
        .layer(Extension(user_session()))
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn installed_identity_is_json_only_and_rejects_non_admin_before_database_access() {
    let body = serde_json::json!({
        "release_id": Uuid::now_v7(),
        "version": "0.8.0",
        "expected_release_updated_at": "2026-08-17T12:34:56.123456Z",
        "product_input_sha256": "a".repeat(64),
        "entries": [],
    });
    let request = Request::builder()
        .method(Method::POST)
        .uri("/installed-identities")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = management_router(service())
        .layer(Extension(user_session()))
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let get = Request::builder()
        .uri("/installed-identities")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = management_router(service())
        .oneshot(get)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[test]
fn server_mounts_download_management_under_admin_and_csrf() {
    let source = include_str!("../../../apps/server/src/app.rs");
    let admin = source
        .split_once("let admin = Router::new()")
        .and_then(|(_, rest)| rest.split_once("let protected = Router::new()"))
        .map(|(block, _)| block)
        .expect("必须能定位管理员 API 路由块");
    assert!(admin.contains("cloud_download::management_router(download_service.clone())"));
    assert!(admin.contains("cloud_auth::require_csrf"));
    assert!(admin.contains("cloud_auth::require_admin"));
    assert!(admin.contains("cloud_auth::authenticate_session"));
}

#[test]
fn administrator_html_has_no_installed_sha_input() {
    let assets = include_str!("../../cloud-web/templates/admin-assets.html");
    let releases = include_str!("../../cloud-web/templates/admin-releases.html");
    for template in [assets, releases] {
        assert!(!template.contains("installed_sha256"));
    }
}

#[tokio::test]
async fn update_router_maps_missing_unknown_and_invalid_queries_to_safe_bad_request() {
    for uri in [
        "/check",
        "/check?platform=windows&architecture=x86_64&package_kind=exe&current_version=7.0.0&extra=value",
        "/check?platform=macos&architecture=x86_64&package_kind=exe&current_version=7.0.0",
        "/check?platform=windows&architecture=x86_64&package_kind=exe&current_version=not-a-version",
        "/check?platform=android&architecture=aarch64&package_kind=zip&current_version=7.0.0",
    ] {
        let request = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
        let response = update_router(service())
            .oneshot(request)
            .await
            .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        let body = to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap_or_else(|error| panic!("读取响应失败: {error}"));
        let value: Value = serde_json::from_slice(&body)
            .unwrap_or_else(|error| panic!("错误响应应为 JSON: {error}"));
        assert_eq!(value["code"], "validation_error");
    }
}

#[tokio::test]
async fn update_router_only_exposes_get_checks_and_stays_separate_from_download_router() {
    let post = Request::builder()
        .method(Method::POST)
        .uri("/check")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = update_router(service())
        .oneshot(post)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    let tauri_post = Request::builder()
        .method(Method::POST)
        .uri("/tauri")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = update_router(service())
        .oneshot(tauri_post)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

    let tauri_invalid = Request::builder()
        .uri("/tauri?architecture=x86_64&package_kind=zip&current_version=0.7.7")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = update_router(service())
        .oneshot(tauri_invalid)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let manifest = Request::builder()
        .uri("/releases")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = update_router(service())
        .oneshot(manifest)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let check = Request::builder()
        .uri("/check?platform=windows&architecture=x86_64&package_kind=exe&current_version=7.0.0")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = public_router(service())
        .oneshot(check)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
