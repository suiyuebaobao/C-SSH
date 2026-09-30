//! 验证站点媒体公开只读路由与管理员写路由保持隔离。

use axum::{
    Extension,
    body::Body,
    http::{Method, Request, StatusCode},
};
use chrono::{Duration, Utc};
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{Service, management_router, public_router};

fn service() -> Service {
    let pool: PgPool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://example:example@example.com/example")
        .expect("固定测试连接串应可创建惰性连接池");
    Service::new(pool, "site-media-test")
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
async fn public_router_rejects_management_writes() {
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("/{}/publish", Uuid::now_v7()))
        .body(Body::empty())
        .expect("测试请求应可构造");
    let response = public_router(service())
        .oneshot(request)
        .await
        .expect("公开路由应返回响应");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn management_router_has_no_public_content_route() {
    let request = Request::builder()
        .uri("/home-qr/content")
        .body(Body::empty())
        .expect("测试请求应可构造");
    let response = management_router(service())
        .oneshot(request)
        .await
        .expect("管理路由应返回响应");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn management_handler_rejects_non_admin_before_database_access() {
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .expect("测试请求应可构造");
    let response = management_router(service())
        .layer(Extension(user_session()))
        .oneshot(request)
        .await
        .expect("管理路由应返回响应");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
