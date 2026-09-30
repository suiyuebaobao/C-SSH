//! 验证 server nest 后不会再次出现重复的版本路径前缀。

use axum::{
    body::Body,
    extract::Extension,
    http::{Request, StatusCode},
};
use chrono::{Duration, Utc};
use cloud_domain::AuthenticatedSession;
use cloud_store::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{Service, router};

fn lazy_pool() -> PgPool {
    let Ok(pool) = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://example:example@example.com/example")
    else {
        panic!("固定测试连接串应可创建惰性连接池");
    };
    pool
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
async fn rejects_duplicated_releases_prefix() {
    let request = Request::builder()
        .uri(format!("/releases/{}", uuid::Uuid::now_v7()))
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = router(Service::new(lazy_pool()))
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn handler_rejects_non_admin_session_before_database_access() {
    let request = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("构造请求失败: {error}"));
    let response = router(Service::new(lazy_pool()))
        .layer(Extension(user_session()))
        .oneshot(request)
        .await
        .unwrap_or_else(|error| panic!("执行路由失败: {error}"));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
