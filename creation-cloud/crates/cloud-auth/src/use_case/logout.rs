//! 删除当前数据库会话，使对应 Cookie 令牌立即失效。

use cloud_domain::AppResult;
use cloud_store::PgPool;

use crate::{repository, session::AuthenticatedSession};

pub(crate) async fn execute(pool: &PgPool, session: &AuthenticatedSession) -> AppResult<()> {
    repository::logout::delete(pool, session.session_id, session.account_id).await
}
