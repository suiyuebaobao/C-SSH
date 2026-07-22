//! 将独立业务模块挂载到统一页面与版本化 API 路由。

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use cloud_config::CloudConfig;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{admin_overview, http_trace, request_id, services::AppServices};

pub fn build(services: AppServices, config: CloudConfig) -> Router {
    let seo = cloud_web::SeoConfig::from_validated_origin(
        config.public_base_url.clone(),
        config.google_site_verification.clone(),
        config.baidu_site_verification.clone(),
    );
    let auth_service = services.auth.clone();
    let admin_service = services.admin.clone();
    let user_service = services.user.clone();
    let device_service = services.device.clone();
    let sync_service = services.sync.clone();
    let model_service = services.model.clone();
    let vault_service = services.vault.clone();
    let download_service = services.download.clone();
    let public_page_state = cloud_web::PublicPageState::new(seo, download_service.clone());
    let console_page_state = cloud_web::ConsolePageState::new(
        auth_service.clone(),
        user_service.clone(),
        device_service.clone(),
        sync_service.clone(),
        model_service.clone(),
        vault_service.clone(),
        download_service.clone(),
    );
    let feedback_service = services.feedback.clone();
    let release_service = services.release.clone();
    let site_media_service = services.site_media.clone();
    let admin_page_state = cloud_web::AdminPageState::new(
        admin_service.clone(),
        release_service.clone(),
        download_service.clone(),
        site_media_service.clone(),
        services.pool.clone(),
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
        .nest(
            "/maintenance",
            cloud_maintenance::management_router(services.maintenance.clone()),
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
        .nest("/users", cloud_user::router(user_service))
        .nest("/devices", cloud_device::router(device_service))
        .nest("/sync", cloud_sync::router(sync_service))
        .nest("/models", cloud_model::router(model_service))
        .nest("/vault", cloud_vault::router(vault_service))
        .nest(
            "/downloads",
            cloud_download::account_router(download_service.clone()),
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

    let console = cloud_web::console_router_with_state(console_page_state)
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::require_page_session,
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

    cloud_web::public_router_with_state(public_page_state)
        .nest("/web/auth", cloud_auth::form_router(auth_service))
        .nest("/console", console)
        .nest("/admin", admin_pages)
        .nest("/api/v1", api)
        .route("/health/live", get(health_live))
        .route(
            "/health/ready",
            get(move || health_ready(ready_state.clone())),
        )
        .layer(middleware::from_fn(noindex_private_routes))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(http_trace::make_span))
        .layer(middleware::from_fn(request_id::attach))
}

async fn noindex_private_routes(request: Request, next: Next) -> Response {
    if route_requires_noindex(request.uri().path()) {
        cloud_web::noindex_response(request, next).await
    } else {
        next.run(request).await
    }
}

fn route_requires_noindex(path: &str) -> bool {
    matches!(path, "/login" | "/en/login" | "/register" | "/en/register")
        || ["/web/auth", "/console", "/admin", "/api/v1", "/health"]
            .iter()
            .any(|prefix| {
                path.strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
            })
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

    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
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
            maintenance: cloud_config::MaintenanceConfig::default(),
        };
        let services = AppServices::new(pool, &config);
        build(services, config)
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

    #[tokio::test]
    async fn maintenance_status_api_uses_the_admin_authentication_chain() {
        let request = Request::builder()
            .uri("/api/v1/admin/maintenance")
            .body(Body::empty())
            .expect("请求应可构造");
        let response = app().oneshot(request).await.expect("应用应返回响应");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn account_download_history_requires_a_session() {
        let request = Request::builder()
            .uri("/api/v1/downloads/history")
            .body(Body::empty())
            .expect("请求应可构造");
        let response = app().oneshot(request).await.expect("应用应返回响应");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn account_download_entry_requires_a_session_before_source_lookup() {
        let asset_id = uuid::Uuid::now_v7();
        let source_id = uuid::Uuid::now_v7();
        let request = Request::builder()
            .uri(format!(
                "/api/v1/downloads/account/assets/{asset_id}/sources/{source_id}"
            ))
            .body(Body::empty())
            .expect("请求应可构造");
        let response = app().oneshot(request).await.expect("应用应返回响应");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn account_pages_and_form_routes_send_noindex_response_header() {
        let app = app();
        for path in ["/login", "/en/login", "/register", "/en/register"] {
            let response = app
                .clone()
                .oneshot(
                    Request::get(path)
                        .body(Body::empty())
                        .expect("账号页面请求应可构造"),
                )
                .await
                .expect("账号页面应返回响应");
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(response.headers()["x-robots-tag"], "noindex, nofollow");
        }

        for path in ["/web/auth/login", "/web/auth/register"] {
            let response = app
                .clone()
                .oneshot(
                    Request::post(path)
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::empty())
                        .expect("空账号表单请求应可构造"),
                )
                .await
                .expect("账号表单路由应返回响应");
            assert_eq!(
                response.status(),
                StatusCode::UNPROCESSABLE_ENTITY,
                "{path}"
            );
            assert_eq!(response.headers()["x-robots-tag"], "noindex, nofollow");
        }
    }

    #[tokio::test]
    async fn public_content_does_not_inherit_account_noindex_header() {
        for path in ["/", "/security", "/downloads"] {
            assert!(!route_requires_noindex(path), "{path}");
        }

        let response = app()
            .oneshot(
                Request::get("/security")
                    .body(Body::empty())
                    .expect("公开页面请求应可构造"),
            )
            .await
            .expect("公开页面应返回响应");
        assert_eq!(response.status(), StatusCode::OK);
        assert!(!response.headers().contains_key("x-robots-tag"));
    }

    #[tokio::test]
    async fn unmatched_private_namespaces_remain_noindex() {
        for path in [
            "/api/v1/seo-not-found",
            "/console/seo-not-found",
            "/admin/seo-not-found",
            "/health/seo-not-found",
        ] {
            let response = app()
                .oneshot(
                    Request::get(path)
                        .body(Body::empty())
                        .expect("私有命名空间请求应可构造"),
                )
                .await
                .expect("私有命名空间应返回响应");
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
            assert_eq!(response.headers()["x-robots-tag"], "noindex, nofollow");
        }
    }
}
