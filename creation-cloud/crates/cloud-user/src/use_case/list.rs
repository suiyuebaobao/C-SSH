//! 以当前账号为固定范围分页列出用户资料。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;

use crate::{Profile, repository};

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    page: PageQuery,
) -> AppResult<Page<Profile>> {
    let page = page.normalized();
    let (items, total) = repository::list::find(pool, session.account_id, page).await?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}
