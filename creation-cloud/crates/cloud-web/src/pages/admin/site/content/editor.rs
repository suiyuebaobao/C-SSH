use cloud_domain::{AppError, AppResult};
use cloud_site_content::SiteContentRevision;
use serde_json::Value;

pub(crate) struct ContentEditor {
    pub(crate) id: String,
    pub(crate) document_key: &'static str,
    pub(crate) document_label: &'static str,
    pub(crate) locale: String,
    pub(crate) locale_label: &'static str,
    pub(crate) revision: i64,
    pub(crate) sections: Vec<EditorSection>,
}

pub(crate) struct EditorSection {
    pub(crate) title: String,
    pub(crate) fields: Vec<EditorField>,
}

pub(crate) struct EditorField {
    pub(crate) path: String,
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) kind: &'static str,
}

impl TryFrom<SiteContentRevision> for ContentEditor {
    type Error = AppError;

    fn try_from(record: SiteContentRevision) -> AppResult<Self> {
        let document_key = record.document_key.as_str();
        let document_label = match document_key {
            "site_shell" => "公共页头与页脚",
            "home" => "首页正文",
            _ => return Err(AppError::Internal("站点内容文档类型无效".into())),
        };
        let locale_label = match record.locale {
            cloud_site::Locale::ZhCn => "简体中文",
            cloud_site::Locale::En => "English",
        };
        let value = serde_json::to_value(&record.content)
            .map_err(|_| AppError::Internal("站点内容无法生成编辑表单".into()))?;
        let sections = flatten(value)?;
        Ok(Self {
            id: record.id.to_string(),
            document_key,
            document_label,
            locale: record.locale.code().to_owned(),
            locale_label,
            revision: record.revision,
            sections,
        })
    }
}

fn flatten(value: Value) -> AppResult<Vec<EditorSection>> {
    let mut fields = Vec::new();
    walk(&value, "", &mut fields)?;
    let mut sections: Vec<EditorSection> = Vec::new();
    for field in fields {
        let title = group_title(&field.path);
        if let Some(section) = sections.iter_mut().find(|section| section.title == title) {
            section.fields.push(field);
        } else {
            sections.push(EditorSection {
                title,
                fields: vec![field],
            });
        }
    }
    Ok(sections)
}

fn walk(value: &Value, path: &str, fields: &mut Vec<EditorField>) -> AppResult<()> {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if is_structural(key) {
                    continue;
                }
                walk(item, &format!("{path}/{key}"), fields)?;
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                walk(item, &format!("{path}/{index}"), fields)?;
            }
        }
        Value::String(text) => fields.push(EditorField {
            path: path.to_owned(),
            label: field_label(path),
            value: text.clone(),
            kind: field_kind(path),
        }),
        Value::Bool(value) => fields.push(EditorField {
            path: path.to_owned(),
            label: field_label(path),
            value: value.to_string(),
            kind: "boolean",
        }),
        Value::Null if path.ends_with("/media_slot") => fields.push(EditorField {
            path: path.to_owned(),
            label: field_label(path),
            value: String::new(),
            kind: "media",
        }),
        Value::Null | Value::Number(_) => {}
    }
    Ok(())
}

fn is_structural(key: &str) -> bool {
    matches!(
        key,
        "document_key" | "schema_version" | "anchor" | "layout" | "tone"
    )
}

fn field_kind(path: &str) -> &'static str {
    if path.ends_with("/media_slot") {
        "media"
    } else if path.ends_with("/href") {
        "link"
    } else if [
        "description",
        "summary",
        "body",
        "answer",
        "lead",
        "note",
        "position",
        "shell",
    ]
    .iter()
    .any(|needle| path.ends_with(&format!("/{needle}")))
    {
        "textarea"
    } else {
        "text"
    }
}

fn group_title(path: &str) -> String {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments.get(1) == Some(&"navigation") {
        return format!(
            "页头导航 {}",
            number_label(segments.get(2).copied().unwrap_or("0"))
        );
    }
    if segments.get(1) == Some(&"footer_navigation") {
        return format!(
            "页脚导航 {}",
            number_label(segments.get(2).copied().unwrap_or("0"))
        );
    }
    if segments.get(2) == Some(&"platforms") {
        return format!(
            "平台 {}",
            number_label(segments.get(3).copied().unwrap_or("0"))
        );
    }
    if segments.get(2) == Some(&"sections") {
        return format!(
            "功能区块 {}",
            number_label(segments.get(3).copied().unwrap_or("0"))
        );
    }
    if segments.get(2) == Some(&"faqs") {
        return format!(
            "FAQ {}",
            number_label(segments.get(3).copied().unwrap_or("0"))
        );
    }
    if path.contains("/qr_widget/") || path.ends_with("/media_slot") {
        return "二维码".into();
    }
    if path.contains("/seo_") {
        return "SEO 主题区".into();
    }
    if path.contains("/final_") {
        return "收尾 CTA".into();
    }
    if path.contains("/actions/") {
        return "首屏按钮".into();
    }
    if segments.get(1) == Some(&"content") {
        "首页总体".into()
    } else {
        "公共页头与页脚".into()
    }
}

fn field_label(path: &str) -> String {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let key = segments.last().copied().unwrap_or("value");
    let translated = match key {
        "label" => "文字",
        "href" => "链接",
        "title" => "标题",
        "heading" => "标题",
        "description" => "说明",
        "lead" => "引导文字",
        "body" => "正文",
        "meta" => "补充信息",
        "question" => "问题",
        "answer" => "回答",
        "planned" => "规划中",
        "media_slot" => "受控媒体",
        "footer_signature" => "页脚签名",
        other => other,
    };
    format!("{translated} · {path}")
}

fn number_label(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(0) + 1
}
