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

use crate::{
    admin_overview, client_config, http_trace, proxy_headers, request_id, services::AppServices,
};

pub fn build(services: AppServices, config: CloudConfig) -> Router {
    let seo = cloud_site_api::SeoConfig::from_validated_origin(
        config.public_base_url.clone(),
        config.google_site_verification.clone(),
        config.baidu_site_verification.clone(),
    );
    let auth_service = services.auth.clone();
    let admin_service = services.admin.clone();
    let announcement_service = services.announcement.clone();
    let user_service = services.user.clone();
    let device_service = services.device.clone();
    let host_service = services.host.clone();
    let model_service = services.model.clone();
    let notification_service = services.notification.clone();
    let proxy_service = services.proxy.clone();
    let download_service = services.download.clone();
    let seo_topic_service = services.seo.clone();
    let site_content_service = services.site_content.clone();
    let public_page_state = cloud_site_api::SiteState::new(
        seo,
        download_service.clone(),
        seo_topic_service.clone(),
        site_content_service.clone(),
    );
    let feedback_service = services.feedback.clone();
    let release_service = services.release.clone();
    let site_media_service = services.site_media.clone();
    let tutorial_service = services.tutorial.clone();
    let health_state = crate::health::HealthState {
        pool: services.pool.clone(),
        download: download_service.clone(),
        site_content: site_content_service.clone(),
        site_media: site_media_service.clone(),
        environment: config.environment.clone(),
    };
    let admin_overview_state = admin_overview::AdminOverviewState::new(
        admin_service.clone(),
        feedback_service.clone(),
        health_state.clone(),
    );
    let admin = Router::new()
        .route("/overview", get(admin_overview::handle))
        .with_state(admin_overview_state)
        .merge(cloud_auth::settings_router(auth_service.clone()))
        .merge(cloud_admin::router_without_overview(admin_service.clone()))
        .nest(
            "/announcements",
            cloud_announcement::management_router(announcement_service.clone()),
        )
        .nest(
            "/users",
            cloud_host::management_router(host_service.clone()),
        )
        .nest(
            "/models",
            cloud_model::management_router(model_service.clone()),
        )
        .nest("/releases", cloud_release::router(release_service.clone()))
        .nest(
            "/downloads",
            cloud_download::management_router(download_service.clone()).merge(
                cloud_download::new_download_router(cloud_download::NewDownloadState::new(
                    download_service.clone(),
                    release_service.clone(),
                )),
            ),
        )
        .nest(
            "/tutorials",
            cloud_tutorial::management_router(tutorial_service.clone()),
        )
        .nest(
            "/site-media",
            cloud_site_media::management_router(site_media_service.clone()),
        )
        .nest(
            "/site-content",
            cloud_site_content::management_router(site_content_service),
        )
        .nest("/seo", cloud_seo::management_router(seo_topic_service))
        .nest(
            "/feedback",
            cloud_feedback::management_router(feedback_service.clone()),
        )
        .nest(
            "/maintenance",
            cloud_maintenance::management_router(services.maintenance.clone()),
        )
        .nest(
            "/notifications",
            cloud_notification::management_router(notification_service.clone()),
        )
        .merge(cloud_device::management_router(device_service.clone()))
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
        .nest("/hosts", cloud_host::host_router(host_service.clone()))
        .nest("/sync", cloud_host::sync_router(host_service))
        .nest(
            "/downloads",
            cloud_download::account_router(download_service.clone()),
        )
        .nest("/feedback", cloud_feedback::user_router(feedback_service))
        .nest(
            "/notifications",
            cloud_notification::account_router(notification_service),
        )
        .nest("/proxy", cloud_proxy::router(proxy_service))
        .layer(middleware::from_fn(cloud_auth::require_csrf))
        .route_layer(middleware::from_fn_with_state(
            auth_service.clone(),
            cloud_auth::require_session,
        ));
    let api = Router::new()
        .nest("/site", cloud_site_api::router(public_page_state))
        .nest(
            "/client",
            client_config::router(client_config::ClientConfigState::new(
                announcement_service.clone(),
                auth_service.clone(),
                download_service.clone(),
            )),
        )
        .nest("/auth", cloud_auth::router(auth_service.clone()))
        .nest(
            "/updates",
            cloud_download::update_router(download_service.clone()),
        )
        .nest(
            "/announcements",
            cloud_announcement::public_router(announcement_service),
        )
        .nest(
            "/downloads",
            cloud_download::public_router(download_service),
        )
        .nest(
            "/site-media",
            cloud_site_media::public_router(site_media_service),
        )
        .nest(
            "/tutorials",
            cloud_tutorial::public_router(tutorial_service),
        )
        .nest("/models", cloud_model::router(model_service))
        .nest("/admin", admin)
        .merge(protected)
        .layer(middleware::from_fn(noindex_response));

    let router = Router::new()
        .nest("/api/v1", api)
        .route("/health/live", get(health_live))
        .route(
            "/health/ready",
            get(move || health_ready(health_state.clone())),
        )
        .layer(middleware::from_fn(noindex_private_routes))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http().make_span_with(http_trace::make_span))
        .layer(middleware::from_fn(proxy_headers::attach))
        .layer(middleware::from_fn(request_id::attach));
    if config
        .intranet
        .as_ref()
        .is_some_and(|intranet| intranet.uses_http())
    {
        router.layer(middleware::from_fn(cloud_auth::intranet_http_cookies))
    } else {
        router
    }
}

async fn noindex_response(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "x-robots-tag",
        axum::http::HeaderValue::from_static("noindex, nofollow"),
    );
    response
}

async fn noindex_private_routes(request: Request, next: Next) -> Response {
    if route_requires_noindex(request.uri().path()) {
        noindex_response(request, next).await
    } else {
        next.run(request).await
    }
}

fn route_requires_noindex(path: &str) -> bool {
    matches!(
        path,
        "/login"
            | "/en/login"
            | "/register"
            | "/en/register"
            | "/verify-email"
            | "/en/verify-email"
            | "/verify-login"
            | "/en/verify-login"
            | "/forgot-password"
            | "/en/forgot-password"
            | "/reset-password"
            | "/en/reset-password"
    ) || ["/web/auth", "/console", "/admin", "/api/v1", "/health"]
        .iter()
        .any(|prefix| {
            path.strip_prefix(prefix)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('/'))
        })
}

async fn health_live() -> impl IntoResponse {
    health_response(StatusCode::OK, "ok")
}

async fn health_ready(state: crate::health::HealthState) -> impl IntoResponse {
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

