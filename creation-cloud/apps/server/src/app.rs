//! 将独立业务模块挂载到统一页面与版本化 API 路由。

use axum::{Router, http::StatusCode, middleware, response::IntoResponse, routing::get};
use cloud_config::CloudConfig;
use cloud_store::PgPool;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{admin_overview, http_trace, request_id};

pub fn build(pool: PgPool, config: CloudConfig) -> Router {
    let seo = cloud_web::SeoConfig::from_validated_origin(
        config.public_base_url.clone(),
        config.google_site_verification.clone(),
        config.baidu_site_verification.clone(),
    );
    let auth_service = cloud_auth::Service::new(pool.clone(), config.session_ttl);
    let admin_service = cloud_admin::Service::new(pool.clone());
    let download_service = cloud_download::Service::new(pool.clone(), config.download_root.clone());
    let feedback_service = cloud_feedback::Service::new(pool.clone());
    let release_service = cloud_release::Service::new(pool.clone());
    let site_media_service =
        cloud_site_media::Service::new(pool.clone(), config.site_media_root.clone());
    let admin_page_state = cloud_web::AdminPageState::new(
        admin_service.clone(),
        release_service.clone(),
        download_service.clone(),
        site_media_service.clone(),
        pool.clone(),
        config.environment.clone(),
    );
    let admin_overview_state = admin_overview::AdminOverviewState::new(
        admin_service.clone(),
        feedback_service.clone(),
        admin_page_state.clone(),
    );
    let admin = Router::new()
        .route("/overview", get(admin_overview::handle))
        .with_state(admin_overview_state)
        .merge(cloud_admin::router_without_overview(admin_service.clone()))
        .nest("/releases", cloud_release::router(release_service))
        .nest(
            "/downloads",
            cloud_download::management_router(download_service.clone()),
        )
        .nest(
            "/site-media",
            cloud_site_media::management_router(site_media_service.clone()),
        )
        .nest(
            "/feedback",
            cloud_feedback::management_router(feedback_service.clone()),
        )
        .layer(middleware::from_fn(cloud_auth::require_csrf))
        .layer(middleware::from_fn_with_state(
            admin_service.clone(),
            cloud_admin::audit_write_requests,
        ))
        .route_layer(middleware::from_fn(cloud_auth::require_admin))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::authenticate_session,
        ));
    let protected = Router::new()
        .nest(
            "/users",
            cloud_user::router(cloud_user::Service::new(pool.clone())),
        )
        .nest(
            "/devices",
            cloud_device::router(cloud_device::Service::new(pool.clone())),
        )
        .nest(
            "/sync",
            cloud_sync::router(cloud_sync::Service::new(pool.clone())),
        )
        .nest(
            "/models",
            cloud_model::router(cloud_model::Service::new(pool.clone())),
        )
        .nest(
            "/vault",
            cloud_vault::router(cloud_vault::Service::new(pool.clone())),
        )
        .nest("/feedback", cloud_feedback::user_router(feedback_service))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::require_session,
        ));
    let api = Router::new()
        .nest("/auth", cloud_auth::router(auth_service.clone()))
        .nest(
            "/downloads",
            cloud_download::public_router(download_service),
        )
        .nest(
            "/site-media",
            cloud_site_media::public_router(site_media_service),
        )
        .nest("/admin", admin)
        .merge(protected)
        .layer(middleware::from_fn(cloud_web::noindex_response));

    let console = cloud_web::console_router()
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::require_session,
        ))
        .layer(middleware::from_fn(cloud_web::noindex_response));
    let admin_pages = cloud_web::admin_router_with_state(admin_page_state.clone())
        .layer(middleware::from_fn(cloud_auth::require_csrf))
        .layer(middleware::from_fn_with_state(
            admin_service,
            cloud_admin::audit_write_requests,
        ))
        .route_layer(middleware::from_fn(cloud_auth::require_admin))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::authenticate_page_session,
        ))
        .layer(middleware::from_fn(cloud_web::noindex_response));

    let ready_state = admin_page_state.clone();

    cloud_web::public_router_with_seo(seo)
        .nest("/web/auth", cloud_auth::form_router(auth_service))
        .nest("/console", console)
        .nest("/admin", admin_pages)
        .nest("/api/v1", api)
        .route("/health/live", get(health_live))
        .route(
            "/health/ready",
            get(move || health_ready(ready_state.clone())),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(http_trace::make_span))
        .layer(middleware::from_fn(request_id::attach))
}

async fn health_live() -> impl IntoResponse {
    health_response(StatusCode::OK, "ok")
}

async fn health_ready(state: cloud_web::AdminPageState) -> impl IntoResponse {
    let ready = state.health().await.ready;
    if ready {
        health_response(StatusCode::OK, "ok")
    } else {
        health_response(StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    }
}

fn health_response(status: StatusCode, body: &'static str) -> impl IntoResponse {
    (
        [
            ("x-robots-tag", "noindex, nofollow"),
            ("content-type", "text/plain; charset=utf-8"),
        ],
        (status, body),
    )
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    use super::*;

    fn app() -> Router {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://example:example@example.com/example")
            .expect("固定测试连接串应可创建惰性连接池");
        let config = CloudConfig {
            bind_addr: "127.0.0.1:8088".parse().expect("地址应有效"),
            database_url: "postgres://example:example@example.com/example".to_owned(),
            public_base_url: "http://127.0.0.1:8088".parse().expect("URL 应有效"),
            google_site_verification: None,
            baidu_site_verification: None,
            download_root: PathBuf::from("data/downloads"),
            site_media_root: PathBuf::from("data/site-media"),
            session_ttl: Duration::from_secs(3600),
            environment: "development".to_owned(),
        };
        build(pool, config)
    }

    #[tokio::test]
    async fn feedback_api_requires_a_session() {
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/feedback")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("请求应可构造");
        let response = app().oneshot(request).await.expect("应用应返回响应");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn feedback_management_api_uses_the_admin_authentication_chain() {
        let request = Request::builder()
            .uri("/api/v1/admin/feedback")
            .body(Body::empty())
            .expect("请求应可构造");
        let response = app().oneshot(request).await.expect("应用应返回响应");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
