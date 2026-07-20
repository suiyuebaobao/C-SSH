//! 处理登录与注册的 SSR 表单页面，并收敛登录后的站内返回地址。

use axum::{extract::Query, response::Html};
use cloud_domain::AppResult;
use cloud_site::{Locale, PageId};
use serde::Deserialize;

use crate::render;

#[derive(Default, Deserialize)]
pub(crate) struct LoginQuery {
    next: Option<String>,
}

pub(crate) async fn login(Query(query): Query<LoginQuery>) -> AppResult<Html<String>> {
    render::account(PageId::Login, Locale::ZhCn, safe_next(query.next))
}

pub(crate) async fn login_en(Query(query): Query<LoginQuery>) -> AppResult<Html<String>> {
    render::account(PageId::Login, Locale::En, safe_next(query.next))
}

pub(crate) async fn register() -> AppResult<Html<String>> {
    render::account(PageId::Register, Locale::ZhCn, None)
}

pub(crate) async fn register_en() -> AppResult<Html<String>> {
    render::account(PageId::Register, Locale::En, None)
}

fn safe_next(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_owned();
    let allowed_root = value == "/admin"
        || value.starts_with("/admin/")
        || value == "/console"
        || value.starts_with("/console/")
        || value == "/feedback"
        || value == "/en/feedback";
    if !allowed_root
        || value.len() > 256
        || value.starts_with("//")
        || value.contains(['\\', '#'])
        || value.chars().any(char::is_control)
    {
        return None;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::safe_next;

    #[test]
    fn keeps_only_known_local_workspace_destinations() {
        assert_eq!(
            safe_next(Some("/admin/releases?lang=en".to_owned())),
            Some("/admin/releases?lang=en".to_owned())
        );
        assert_eq!(safe_next(Some("https://example.com".to_owned())), None);
        assert_eq!(safe_next(Some("//example.com/admin".to_owned())), None);
        assert_eq!(safe_next(Some("/downloads".to_owned())), None);
        assert_eq!(
            safe_next(Some("/en/feedback".to_owned())),
            Some("/en/feedback".to_owned())
        );
    }
}
