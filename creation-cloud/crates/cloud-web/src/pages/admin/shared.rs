//! 提供管理后台页面共用的会话派生、语言、渲染与 HTMX 响应规则。
//! 所有写请求仍调用领域 Service；此处只把结果转换为可读 HTML 或安全跳转。

use askama::Template;
use axum::{
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession, PageQuery};
use cloud_site::{Locale, PageId, SiteView, content_service};
use serde::Deserialize;

use crate::seo::SeoHead;

const HX_REQUEST: &str = "hx-request";
const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");

pub(crate) struct PageParts {
    pub(crate) view: SiteView,
    pub(crate) seo: SeoHead,
    pub(crate) session_identity: String,
    pub(crate) csrf_token: String,
    pub(crate) is_en: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct AdminListQuery {
    pub(crate) lang: Option<String>,
    pub(crate) page: Option<u32>,
    pub(crate) size: Option<u32>,
}

impl AdminListQuery {
    pub(crate) fn locale(&self) -> Locale {
        locale(self.lang.as_deref())
    }

    pub(crate) fn page_query(&self) -> PageQuery {
        PageQuery {
            page: self.page.unwrap_or(1),
            size: self.size.unwrap_or(20),
        }
        .normalized()
    }
}

#[derive(Template)]
#[template(path = "admin-feedback.html")]
struct FeedbackTemplate<'a> {
    title: &'a str,
    message: &'a str,
}

pub(crate) fn page_parts(
    page: PageId,
    locale: Locale,
    session: &AuthenticatedSession,
) -> PageParts {
    PageParts {
        view: content_service().view(page, locale),
        seo: SeoHead::private(),
        session_identity: session
            .admin_login_name
            .clone()
            .unwrap_or_else(|| session.email.clone()),
        csrf_token: session.csrf_token.clone(),
        is_en: locale == Locale::En,
    }
}

pub(crate) fn actor_from_session(session: &AuthenticatedSession) -> AppResult<AdminActor> {
    AdminActor::from_session(session)
}

pub(crate) fn locale(value: Option<&str>) -> Locale {
    Locale::from_code(value)
}

pub(crate) fn render(template: &impl Template) -> AppResult<Html<String>> {
    template
        .render()
        .map(Html)
        .map_err(|_| AppError::Internal("管理页面暂时无法渲染".to_owned()))
}

pub(crate) fn action_error(locale: Locale, error: AppError) -> Response {
    let status = error_status(&error);
    let title = if locale == Locale::En {
        "Action failed"
    } else {
        "操作未完成"
    };
    let fallback = if locale == Locale::En {
        "The request could not be completed. Review the input and try again."
    } else {
        "请求未能完成，请检查输入后重试。"
    };
    let detail = match (locale, error) {
        (Locale::En, AppError::Validation(_)) => {
            "The submitted values are invalid. Review the form and try again.".to_owned()
        }
        (Locale::En, AppError::Unauthorized(_)) => {
            "Your session is no longer valid. Sign in again.".to_owned()
        }
        (Locale::En, AppError::Forbidden(_)) => {
            "This administrator is not allowed to perform that action.".to_owned()
        }
        (Locale::En, AppError::NotFound(_)) => "The requested record no longer exists.".to_owned(),
        (Locale::En, AppError::Conflict(_)) => {
            "The action conflicts with the current record state.".to_owned()
        }
        (_, AppError::Unavailable(_) | AppError::Storage(_) | AppError::Internal(_)) => {
            fallback.to_owned()
        }
        (_, other) => other.to_string(),
    };
    match (FeedbackTemplate {
        title,
        message: &detail,
    })
    .render()
    {
        Ok(body) => (status, Html(body)).into_response(),
        Err(_) => (status, Html(fallback.to_owned())).into_response(),
    }
}

pub(crate) fn action_success(headers: &HeaderMap, path: &str, locale: Locale) -> Response {
    let target = localized_admin_path(path, locale);
    if headers
        .get(HX_REQUEST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
    {
        let mut response = StatusCode::OK.into_response();
        if let Ok(value) = HeaderValue::from_str(&target) {
            response.headers_mut().insert(HX_REDIRECT, value);
        }
        response
    } else {
        Redirect::to(&target).into_response()
    }
}

pub(crate) fn localized_admin_path(path: &str, locale: Locale) -> String {
    if locale == Locale::En {
        let separator = if path.contains('?') { '&' } else { '?' };
        format!("{path}{separator}lang=en")
    } else {
        path.to_owned()
    }
}

pub(crate) fn optional_text(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

pub(crate) fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .filter(|item| !item.trim().is_empty())
        .map(|item| item.parse().map_err(serde::de::Error::custom))
        .transpose()
}

fn error_status(error: &AppError) -> StatusCode {
    match error {
        AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
        AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        AppError::Forbidden(_) => StatusCode::FORBIDDEN,
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Conflict(_) | AppError::SyncResyncRequired(_) => StatusCode::CONFLICT,
        AppError::RateLimited(_) | AppError::RateLimitedAfter { .. } => {
            StatusCode::TOO_MANY_REQUESTS
        }
        AppError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        AppError::Storage(_) | AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
