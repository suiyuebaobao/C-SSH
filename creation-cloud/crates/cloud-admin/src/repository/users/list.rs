//! 按有界筛选和稳定顺序分页读取管理用户投影。

use cloud_domain::{AppResult, Page};
use cloud_store::PgPool;
use sqlx::FromRow;

use crate::{
    AdminUser, model::AdminUserListFilter, model::AdminUserRow, repository::map_read_error,
};

#[derive(FromRow)]
struct CountRow {
    total: i64,
}

pub(crate) const COUNT_SQL: &str = r#"
    SELECT count(*)::BIGINT AS total
    FROM accounts
    LEFT JOIN user_profiles ON user_profiles.account_id = accounts.id
    WHERE (
            $1::TEXT IS NULL
            OR lower(COALESCE(accounts.email, '')) ILIKE $1 ESCAPE '\'
            OR lower(COALESCE(accounts.admin_login_name, '')) ILIKE $1 ESCAPE '\'
            OR lower(COALESCE(user_profiles.display_name, '')) ILIKE $1 ESCAPE '\'
          )
      AND ($2::TEXT IS NULL OR lower(accounts.email) = $2)
      AND ($3::TEXT IS NULL OR accounts.role = $3)
      AND ($4::TEXT IS NULL OR accounts.status = $4)
"#;

pub(crate) const LIST_SQL: &str = r#"
    SELECT accounts.id, accounts.email, accounts.admin_login_name,
           accounts.email_verified_at IS NOT NULL AS email_verified,
           COALESCE(user_profiles.display_name, '') AS display_name,
           accounts.role, accounts.status,
           (SELECT count(*)::BIGINT FROM devices WHERE devices.account_id = accounts.id)
               AS device_count,
           (SELECT count(*)::BIGINT FROM cloud_hosts
            WHERE cloud_hosts.account_id = accounts.id
              AND cloud_hosts.is_deleted = FALSE) AS host_count,
           accounts.created_at, accounts.updated_at
    FROM accounts
    LEFT JOIN user_profiles ON user_profiles.account_id = accounts.id
    WHERE (
            $1::TEXT IS NULL
            OR lower(COALESCE(accounts.email, '')) ILIKE $1 ESCAPE '\'
            OR lower(COALESCE(accounts.admin_login_name, '')) ILIKE $1 ESCAPE '\'
            OR lower(COALESCE(user_profiles.display_name, '')) ILIKE $1 ESCAPE '\'
          )
      AND ($2::TEXT IS NULL OR lower(accounts.email) = $2)
      AND ($3::TEXT IS NULL OR accounts.role = $3)
      AND ($4::TEXT IS NULL OR accounts.status = $4)
    ORDER BY accounts.created_at DESC, accounts.id DESC
    LIMIT $5 OFFSET $6
"#;

pub(crate) async fn execute(
    pool: &PgPool,
    filter: &AdminUserListFilter,
) -> AppResult<Page<AdminUser>> {
    let role = filter.role.map(|value| value.as_str());
    let status = filter.status.map(|value| value.as_str());
    let search_pattern = filter.search.as_deref().map(search_pattern);
    let count = sqlx::query_as::<_, CountRow>(COUNT_SQL)
        .bind(search_pattern.as_deref())
        .bind(filter.email.as_deref())
        .bind(role)
        .bind(status)
        .fetch_one(pool)
        .await
        .map_err(map_read_error)?;
    let rows = sqlx::query_as::<_, AdminUserRow>(LIST_SQL)
        .bind(search_pattern.as_deref())
        .bind(filter.email.as_deref())
        .bind(role)
        .bind(status)
        .bind(i64::from(filter.page.size))
        .bind(filter.page.offset())
        .fetch_all(pool)
        .await
        .map_err(map_read_error)?;
    let items = rows
        .into_iter()
        .map(AdminUser::try_from)
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Page {
        items,
        page: filter.page.page,
        size: filter.page.size,
        total: count.total,
    })
}

fn search_pattern(value: &str) -> String {
    let escaped = value
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_");
    format!("%{escaped}%")
}

#[cfg(test)]
mod tests {
    use super::search_pattern;

    #[test]
    fn search_pattern_escapes_like_wildcards() {
        assert_eq!(search_pattern(r"a_b%c\d"), r"%a\_b\%c\\d%");
    }
}
