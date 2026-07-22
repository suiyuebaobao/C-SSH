//! 装配 Creation Cloud 的 SSR 页面与内嵌静态资源路由。

mod admin_state;
mod assets;
mod console_state;
mod indexing;
mod language_redirect;
mod pages;
mod private_indexing;
mod public_state;
mod query;
mod render;
mod seo;

use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
};

pub use admin_state::{AdminHealth, AdminPageState};
pub use console_state::ConsolePageState;
pub use private_indexing::noindex_response;
pub use public_state::PublicPageState;
pub use seo::SeoConfig;

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn public_router() -> Router {
    public_router_with_seo(SeoConfig::default())
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn public_router_with_seo(seo: SeoConfig) -> Router {
    Router::new()
        .route("/", get(pages::public::home))
        .route("/en", get(pages::public::home_en))
        .route("/downloads", get(pages::public::downloads))
        .route("/en/downloads", get(pages::public::downloads_en))
        .route("/changelog", get(pages::public::changelog))
        .route("/en/changelog", get(pages::public::changelog_en))
        .route("/robots.txt", get(indexing::robots))
        .route("/sitemap.xml", get(indexing::sitemap))
        .merge(public_common_router::<SeoConfig>())
        .route_layer(middleware::from_fn(language_redirect::canonicalize))
        .with_state(seo)
        .merge(assets::router())
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn public_router_with_state(state: PublicPageState) -> Router {
    Router::new()
        .route("/", get(pages::public::home_live))
        .route("/en", get(pages::public::home_en_live))
        .route("/downloads", get(pages::public::downloads_live))
        .route("/en/downloads", get(pages::public::downloads_en_live))
        .route("/changelog", get(pages::public::changelog_live))
        .route("/en/changelog", get(pages::public::changelog_en_live))
        .route("/robots.txt", get(indexing::robots))
        .route("/sitemap.xml", get(indexing::sitemap_live))
        .merge(public_common_router::<PublicPageState>())
        .route_layer(middleware::from_fn(language_redirect::canonicalize))
        .with_state(state)
        .merge(assets::router())
}

fn public_common_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    SeoConfig: axum::extract::FromRef<S>,
{
    Router::new()
        .route("/en/", get(language_redirect::english_root_slash))
        .route("/features", get(language_redirect::legacy_documentation))
        .route(
            "/en/features",
            get(language_redirect::legacy_documentation_en),
        )
        .route("/tutorials", get(language_redirect::legacy_documentation))
        .route(
            "/en/tutorials",
            get(language_redirect::legacy_documentation_en),
        )
        .route("/platforms", get(language_redirect::legacy_documentation))
        .route(
            "/en/platforms",
            get(language_redirect::legacy_documentation_en),
        )
        .route("/docs/getting-started", get(pages::documentation::page))
        .route(
            "/en/docs/getting-started",
            get(pages::documentation::page_en),
        )
        .route("/security", get(pages::public::security))
        .route("/en/security", get(pages::public::security_en))
        .route("/faq", get(pages::public::faq))
        .route("/en/faq", get(pages::public::faq_en))
        .route("/feedback", get(pages::feedback::page))
        .route("/en/feedback", get(pages::feedback::page_en))
        .route("/login", get(pages::account::login))
        .route("/en/login", get(pages::account::login_en))
        .route("/register", get(pages::account::register))
        .route("/en/register", get(pages::account::register_en))
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn console_router() -> Router {
    Router::new()
        .route("/", get(pages::console_preview::overview))
        .route("/profile", get(pages::console_preview::profile))
        .route("/devices", get(pages::console_preview::devices))
        .route("/sync", get(pages::console_preview::sync))
        .route("/models", get(pages::console_preview::models))
        .route("/vault", get(pages::console_preview::vault))
        .route("/downloads", get(pages::console_preview::downloads))
}

#[must_use = "路由必须挂载到已注入认证会话的服务端才会生效"]
pub fn console_router_with_state(state: ConsolePageState) -> Router {
    Router::new()
        .route("/", get(pages::console::overview))
        .route(
            "/profile",
            get(pages::console::profile).post(pages::console::update_profile),
        )
        .route(
            "/profile/password",
            post(pages::console::change_password).layer(DefaultBodyLimit::max(4 * 1024)),
        )
        .route("/devices", get(pages::console::devices))
        .route("/devices/{device_id}", post(pages::console::rename_device))
        .route(
            "/devices/{device_id}/revoke",
            post(pages::console::revoke_device),
        )
        .route("/sync", get(pages::console::sync))
        .route(
            "/models",
            get(pages::console::models).post(pages::console::create_model),
        )
        .route("/models/{model_id}", post(pages::console::update_model))
        .route(
            "/models/{model_id}/delete",
            post(pages::console::delete_model),
        )
        .route("/vault", get(pages::console::vault))
        .route("/downloads", get(pages::console::downloads))
        .with_state(state)
}

#[must_use = "路由必须挂载到服务端才会生效"]
pub fn admin_router() -> Router {
    Router::new()
        .route("/", get(pages::admin::static_overview))
        .route("/users", get(pages::admin::static_users))
        .route("/devices", get(pages::admin::static_devices))
        .route("/releases", get(pages::admin::static_releases))
        .route("/assets", get(pages::admin::static_assets))
        .route("/site", get(pages::admin::static_site))
        .route("/audit", get(pages::admin::static_audit))
        .route("/feedback", get(pages::admin::static_feedback))
}

#[must_use = "路由必须挂载到已注入认证会话的服务端才会生效"]
pub fn admin_router_with_state(state: AdminPageState) -> Router {
    Router::new()
        .route("/", get(pages::admin::overview::page))
        .route("/users", get(pages::admin::users::page))
        .route(
            "/users/{account_id}",
            post(pages::admin::users::update::handle),
        )
        .route("/devices", get(pages::admin::devices::page))
        .route(
            "/devices/{device_id}/revoke",
            post(pages::admin::devices::revoke::handle),
        )
        .route(
            "/releases",
            get(pages::admin::releases::page).post(pages::admin::releases::create::handle),
        )
        .route(
            "/releases/{release_id}",
            post(pages::admin::releases::update::handle),
        )
        .route(
            "/releases/{release_id}/delete",
            post(pages::admin::releases::delete::handle),
        )
        .route(
            "/assets",
            get(pages::admin::assets::page).post(pages::admin::assets::create::handle),
        )
        .route(
            "/assets/{asset_id}",
            post(pages::admin::assets::update::handle),
        )
        .route(
            "/assets/{asset_id}/delete",
            post(pages::admin::assets::delete::handle),
        )
        .route(
            "/assets/{asset_id}/upload",
            post(pages::admin::assets::upload::handle).layer(DefaultBodyLimit::disable()),
        )
        .route(
            "/assets/{asset_id}/sources",
            post(pages::admin::assets::source_create::handle),
        )
        .route(
            "/sources/{source_id}",
            post(pages::admin::assets::source_update::handle),
        )
        .route(
            "/sources/{source_id}/delete",
            post(pages::admin::assets::source_delete::handle),
        )
        .route(
            "/site",
            get(pages::admin::site::page)
                .post(pages::admin::site::create::handle)
                .layer(DefaultBodyLimit::max(2 * 1024 * 1024 + 64 * 1024)),
        )
        .route("/site/{media_id}", post(pages::admin::site::update::handle))
        .route(
            "/site/{media_id}/publish",
            post(pages::admin::site::publish::handle),
        )
        .route(
            "/site/{media_id}/revoke",
            post(pages::admin::site::revoke::handle),
        )
        .route(
            "/site/{media_id}/delete",
            post(pages::admin::site::delete::handle),
        )
        .route("/audit", get(pages::admin::audit::page))
        .route("/feedback", get(pages::admin::feedback::page))
        .route(
            "/feedback/{feedback_id}/status",
            post(pages::admin::feedback::status::handle),
        )
        .route(
            "/feedback/{feedback_id}/redact",
            post(pages::admin::feedback::redact::handle),
        )
        .with_state(state)
}
