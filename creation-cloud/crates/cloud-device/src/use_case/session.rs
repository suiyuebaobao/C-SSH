use cloud_domain::{AppError, AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{SessionView, repository};

pub(crate) async fn list_self(
    pool: &PgPool,
    session: &AuthenticatedSession,
    page: PageQuery,
) -> AppResult<Page<SessionView>> {
    let page = page.normalized();
    let (items, total) =
        repository::session::list_for_account(pool, session.account_id, session.session_id, page)
            .await?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn revoke_self(
    pool: &PgPool,
    session: &AuthenticatedSession,
    session_id: Uuid,
) -> AppResult<bool> {
    let revoked = repository::session::revoke_for_account(
        pool,
        session.account_id,
        session.session_id,
        session_id,
    )
    .await?;
    if !revoked {
        return Err(AppError::NotFound("会话不存在或已撤销".to_owned()));
    }
    Ok(true)
}

pub(crate) async fn list_admin(
    pool: &PgPool,
    session: &AuthenticatedSession,
    account_id: Option<Uuid>,
    page: PageQuery,
) -> AppResult<Page<SessionView>> {
    require_admin(session)?;
    let page = page.normalized();
    let (items, total) = match account_id {
        Some(account_id) => {
            repository::session::list_for_account(pool, account_id, session.session_id, page)
                .await?
        }
        None => repository::session::list_all(pool, session.session_id, page).await?,
    };
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}

pub(crate) async fn delete_admin(
    pool: &PgPool,
    session: &AuthenticatedSession,
    session_id: Uuid,
) -> AppResult<bool> {
    require_admin(session)?;
    let deleted = repository::session::delete_any(pool, session.account_id, session_id).await?;
    if !deleted {
        return Err(AppError::NotFound("会话不存在".to_owned()));
    }
    Ok(true)
}

fn require_admin(session: &AuthenticatedSession) -> AppResult<()> {
    if session.role != "admin" {
        return Err(AppError::Forbidden("需要管理员权限".to_owned()));
    }
    Ok(())
}
