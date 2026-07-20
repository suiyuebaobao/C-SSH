//! 以当前账号为固定范围分页列出设备。

use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppResult, Page, PageQuery};
use cloud_store::PgPool;

use crate::{Device, repository};

pub(crate) async fn execute(
    pool: &PgPool,
    session: &AuthenticatedSession,
    page: PageQuery,
) -> AppResult<Page<Device>> {
    let page = page.normalized();
    let (items, total) = repository::list::find(pool, session.account_id, page).await?;
    Ok(Page {
        items,
        page: page.page,
        size: page.size,
        total,
    })
}
