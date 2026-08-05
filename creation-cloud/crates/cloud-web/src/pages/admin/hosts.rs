//! Compatibility redirect for the former standalone host page.

use axum::{
    extract::{Path, Query},
    response::Redirect,
};
use uuid::Uuid;

use super::shared::AdminListQuery;

pub(crate) async fn page(
    Path(account_id): Path<Uuid>,
    Query(query): Query<AdminListQuery>,
) -> Redirect {
    let language = if query.locale() == cloud_site::Locale::En {
        "&lang=en"
    } else {
        ""
    };
    Redirect::permanent(&format!("/admin/users/{account_id}?tab=hosts{language}"))
}
