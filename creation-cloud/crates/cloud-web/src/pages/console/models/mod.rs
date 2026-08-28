//! 用户只读全局模型目录；模型配置由管理员统一维护。

use askama::Template;
use axum::{Extension, extract::Query, extract::State, response::Html};
use cloud_domain::{AppResult, AuthenticatedSession};
use cloud_site::{PageId, SiteView};

use crate::{ConsolePageState, query::LocaleQuery, seo::SeoHead};

use super::common;

struct ConsoleModelRow {
    name: String,
    provider: String,
    context_length: i32,
    interfaces: Vec<ConsoleModelInterface>,
}

struct ConsoleModelInterface {
    api_format: &'static str,
    model_name: String,
    base_url: String,
}

impl From<cloud_model::PublicGlobalModel> for ConsoleModelRow {
    fn from(value: cloud_model::PublicGlobalModel) -> Self {
        Self {
            name: value.name,
            provider: value.provider,
            context_length: value.context_length,
            interfaces: value
                .interfaces
                .into_iter()
                .map(|item| ConsoleModelInterface {
                    api_format: match item.api_format.as_str() {
                        "anthropic_compatible" => "Claude / Anthropic",
                        "responses_compatible" => "Responses",
                        _ => "OpenAI",
                    },
                    model_name: item.model_name,
                    base_url: item.base_url,
                })
                .collect(),
        }
    }
}

#[derive(Template)]
#[template(path = "console-models.html")]
struct ModelsTemplate {
    view: SiteView,
    seo: SeoHead,
    csrf_token: String,
    is_en: bool,
    models: Vec<ConsoleModelRow>,
    total: i64,
}

pub(crate) async fn page(
    State(state): State<ConsolePageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<LocaleQuery>,
) -> AppResult<Html<String>> {
    let models = state.model().list_public(common::first_page()).await?;
    let locale = query.locale();
    common::render(&ModelsTemplate {
        view: common::view(PageId::Models, locale),
        seo: common::seo(),
        csrf_token: session.csrf_token,
        is_en: common::is_en(locale),
        models: models
            .items
            .into_iter()
            .map(ConsoleModelRow::from)
            .collect(),
        total: models.total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_site::{Locale, content_service};

    #[test]
    fn template_keeps_ai_credentials_out_of_the_catalog() {
        for (locale, expected) in [
            (Locale::ZhCn, "手动同步时也只上传客户端加密的不透明账户记录"),
            (Locale::En, "client-encrypted opaque account records"),
        ] {
            let body = ModelsTemplate {
                view: content_service().view(PageId::Models, locale),
                seo: SeoHead::private(),
                csrf_token: "csrf-example".to_owned(),
                is_en: locale == Locale::En,
                models: Vec::new(),
                total: 0,
            }
            .render()
            .expect("模型目录模板应可渲染");

            assert!(body.contains(expected));
            assert!(!body.contains("API Key 只能作为保险库密文引用"));
            assert!(
                !body
                    .to_ascii_lowercase()
                    .contains("api keys remain vault ciphertext references")
            );
        }

        let body = ModelsTemplate {
            view: content_service().view(PageId::Models, Locale::ZhCn),
            seo: SeoHead::private(),
            csrf_token: "csrf-example".to_owned(),
            is_en: false,
            models: vec![ConsoleModelRow {
                name: "Example".to_owned(),
                provider: "Example".to_owned(),
                context_length: 128_000,
                interfaces: Vec::new(),
            }],
            total: 1,
        }
        .render()
        .expect("模型目录应渲染模型名称");
        assert!(body.contains("<strong>Example</strong>"));
        assert!(!body.contains("可控思考"));
        assert!(!body.contains("Controllable thinking"));
    }
}
