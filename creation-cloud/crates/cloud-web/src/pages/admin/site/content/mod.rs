//! 管理首页与公共站点壳的双语结构化内容。
//! HTML 表单只投影领域文档；校验、CAS、发布和历史不可变性仍由领域服务负责。

mod editor;
mod forms;

use std::collections::HashMap;

use axum::{
    Extension, Form,
    extract::{Path, State},
    http::HeaderMap,
    response::{Html, Response},
};
use cloud_domain::{AppError, AuthenticatedSession};
use cloud_site::{Locale, PageId};
use cloud_site_content::{
    CreateSiteContentInput, SiteContentDocumentKey, SiteContentPayload, SiteContentTransitionInput,
    UpdateSiteContentInput,
};
use uuid::Uuid;

use crate::{AdminPageState, render};

use super::super::shared;

pub(crate) use editor::ContentEditor;
pub(crate) use forms::{CreateContentForm, TransitionForm};

pub(crate) async fn create(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    Form(form): Form<CreateContentForm>,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = CreateSiteContentInput {
        document_key: form.document_key,
        locale: form.content_locale,
        content: None,
    };
    match state.site_content().create_draft(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

pub(crate) async fn update(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    headers: HeaderMap,
    Form(fields): Form<HashMap<String, String>>,
) -> Response {
    let locale = forms::locale(&fields);
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let result = async {
        let expected_revision = forms::expected_revision(&fields)?;
        let current = state.site_content().get(&actor, content_id).await?;
        let content = forms::apply(&current.content, &fields)?;
        state
            .site_content()
            .update_draft(
                &actor,
                content_id,
                UpdateSiteContentInput {
                    expected_revision,
                    content,
                },
            )
            .await
    }
    .await;
    match result {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

pub(crate) async fn publish(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<TransitionForm>,
) -> Response {
    transition(
        state,
        session,
        content_id,
        headers,
        form,
        Transition::Publish,
    )
    .await
}

pub(crate) async fn revoke(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<TransitionForm>,
) -> Response {
    transition(
        state,
        session,
        content_id,
        headers,
        form,
        Transition::Revoke,
    )
    .await
}

pub(crate) async fn rollback(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<TransitionForm>,
) -> Response {
    transition(
        state,
        session,
        content_id,
        headers,
        form,
        Transition::Rollback,
    )
    .await
}

pub(crate) async fn delete(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
    headers: HeaderMap,
    Form(form): Form<TransitionForm>,
) -> Response {
    transition(
        state,
        session,
        content_id,
        headers,
        form,
        Transition::Delete,
    )
    .await
}

pub(crate) async fn preview(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Path(content_id): Path<Uuid>,
) -> Result<Html<String>, AppError> {
    let actor = shared::actor_from_session(&session)?;
    let record = state.site_content().get(&actor, content_id).await?;
    let other_key = match record.document_key {
        SiteContentDocumentKey::SiteShell => SiteContentDocumentKey::Home,
        SiteContentDocumentKey::Home => SiteContentDocumentKey::SiteShell,
    };
    let companion = state
        .site_content()
        .published(other_key, record.locale)
        .await?
        .map_or_else(
            || SiteContentPayload::compiled(other_key, record.locale),
            |published| published.content,
        );
    let topics_locale = match record.locale {
        Locale::ZhCn => cloud_seo::SeoLocale::ZhCn,
        Locale::En => cloud_seo::SeoLocale::En,
    };
    let topics = state
        .seo()
        .public_topics(topics_locale)
        .await?
        .into_iter()
        .map(|topic| topic.phrase)
        .collect();
    let mut documents = vec![companion, record.content];
    documents.sort_by_key(|document| match document.key() {
        SiteContentDocumentKey::SiteShell => 0,
        SiteContentDocumentKey::Home => 1,
    });
    render::home_preview(PageId::Home, record.locale, topics, documents)
}

#[derive(Clone, Copy)]
enum Transition {
    Publish,
    Revoke,
    Rollback,
    Delete,
}

async fn transition(
    state: AdminPageState,
    session: AuthenticatedSession,
    content_id: Uuid,
    headers: HeaderMap,
    form: TransitionForm,
    transition: Transition,
) -> Response {
    let locale = shared::locale(form.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = SiteContentTransitionInput {
        expected_revision: form.expected_revision,
    };
    let result = match transition {
        Transition::Publish => state
            .site_content()
            .publish(&actor, content_id, input)
            .await
            .map(drop),
        Transition::Revoke => state
            .site_content()
            .revoke(&actor, content_id, input)
            .await
            .map(drop),
        Transition::Rollback => state
            .site_content()
            .rollback(&actor, content_id, input)
            .await
            .map(drop),
        Transition::Delete => {
            state
                .site_content()
                .delete_draft(&actor, content_id, input)
                .await
        }
    };
    match result {
        Ok(()) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}
