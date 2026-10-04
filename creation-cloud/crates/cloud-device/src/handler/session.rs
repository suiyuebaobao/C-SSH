use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppResult, AuthenticatedSession, Page, PageQuery};
use uuid::Uuid;

use crate::{Service, SessionView, session};

pub(crate) async fn list_self(
    State(service): State<Service>,
    Extension(current): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<SessionView>>> {
    Ok(Json(service.list_sessions(&current, page).await?))
}

pub(crate) async fn revoke_self(
    State(service): State<Service>,
    Extension(current): Extension<AuthenticatedSession>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Response> {
    let _ = service.revoke_session(&current, session_id).await?;
    Ok(deletion_response(session_id, current.session_id))
}

pub(crate) async fn list_admin(
    State(service): State<Service>,
    Extension(current): Extension<AuthenticatedSession>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<SessionView>>> {
    Ok(Json(
        service.admin_list_sessions(&current, None, page).await?,
    ))
}

pub(crate) async fn list_admin_for_user(
    State(service): State<Service>,
    Extension(current): Extension<AuthenticatedSession>,
    Path(account_id): Path<Uuid>,
    Query(page): Query<PageQuery>,
) -> AppResult<Json<Page<SessionView>>> {
    Ok(Json(
        service
            .admin_list_sessions(&current, Some(account_id), page)
            .await?,
    ))
}

pub(crate) async fn delete_admin(
    State(service): State<Service>,
    Extension(current): Extension<AuthenticatedSession>,
    Path(session_id): Path<Uuid>,
) -> AppResult<Response> {
    let _ = service.admin_delete_session(&current, session_id).await?;
    Ok(deletion_response(session_id, current.session_id))
}

fn deletion_response(session_id: Uuid, current_session_id: Uuid) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    if session_id == current_session_id {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, session::clear_cookie());
    }
    response
}
