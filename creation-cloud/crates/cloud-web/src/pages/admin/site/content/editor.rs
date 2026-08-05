use cloud_domain::{AppError, AppResult};
use cloud_site_content::SiteContentRevision;
use serde_json::Value;

pub(crate) struct ContentEditor {
    pub(crate) id: String,
    pub(crate) document_label_zh: &'static str,
    pub(crate) document_label_en: &'static str,
    pub(crate) locale_label_zh: &'static str,
    pub(crate) locale_label_en: &'static str,
    pub(crate) revision: i64,
    pub(crate) sections: Vec<EditorSection>,
}

pub(crate) struct EditorSection {
    pub(crate) title_zh: String,
    pub(crate) title_en: String,
    pub(crate) fields: Vec<EditorField>,
}

pub(crate) struct EditorField {
    pub(crate) path: String,
    pub(crate) label_zh: String,
    pub(crate) label_en: String,
    pub(crate) value: String,
    pub(crate) kind: &'static str,
}

impl TryFrom<SiteContentRevision> for ContentEditor {
    type Error = AppError;

    fn try_from(record: SiteContentRevision) -> AppResult<Self> {
        let document_key = record.document_key.as_str();
        let (document_label_zh, document_label_en) = match document_key {
            "site_shell" => ("公共页头与页脚", "Shared header and footer"),
            "home" => ("首页正文", "Home page"),
            _ => return Err(AppError::Internal("站点内容文档类型无效".into())),
        };
        let (locale_label_zh, locale_label_en) = match record.locale {
            cloud_site::Locale::ZhCn => ("简体中文", "Chinese"),
            cloud_site::Locale::En => ("英文", "English"),
        };
        let value = serde_json::to_value(&record.content)
            .map_err(|_| AppError::Internal("站点内容无法生成编辑表单".into()))?;
        let sections = flatten(value)?;
        Ok(Self {
            id: record.id.to_string(),
            document_label_zh,
            document_label_en,
            locale_label_zh,
            locale_label_en,
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
        let (title_zh, title_en) = group_title(&field.path);
        if let Some(section) = sections
            .iter_mut()
            .find(|section| section.title_zh == title_zh)
        {
            section.fields.push(field);
        } else {
            sections.push(EditorSection {
                title_zh,
                title_en,
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
        Value::String(text) => {
            let (label_zh, label_en) = field_label(path);
            fields.push(EditorField {
                path: path.to_owned(),
                label_zh,
                label_en,
                value: text.clone(),
                kind: field_kind(path),
            });
        }
        Value::Bool(value) => {
            let (label_zh, label_en) = field_label(path);
            fields.push(EditorField {
                path: path.to_owned(),
                label_zh,
                label_en,
                value: value.to_string(),
                kind: "boolean",
            });
        }
        Value::Null if path.ends_with("/media_slot") => {
            let (label_zh, label_en) = field_label(path);
            fields.push(EditorField {
                path: path.to_owned(),
                label_zh,
                label_en,
                value: String::new(),
                kind: "media",
            });
        }
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

fn group_title(path: &str) -> (String, String) {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if segments.get(1) == Some(&"navigation") {
        let number = number_label(segments.get(2).copied().unwrap_or("0"));
        return (
            format!("页头导航 {number}"),
            format!("Header link {number}"),
        );
    }
    if segments.get(1) == Some(&"footer_navigation") {
        let number = number_label(segments.get(2).copied().unwrap_or("0"));
        return (
            format!("页脚导航 {number}"),
            format!("Footer link {number}"),
        );
    }
    if segments.get(2) == Some(&"platforms") {
        let number = number_label(segments.get(3).copied().unwrap_or("0"));
        return (format!("平台 {number}"), format!("Platform {number}"));
    }
    if segments.get(2) == Some(&"sections") {
        let number = number_label(segments.get(3).copied().unwrap_or("0"));
        return (
            format!("功能区块 {number}"),
            format!("Feature section {number}"),
        );
    }
    if segments.get(2) == Some(&"faqs") {
        let number = number_label(segments.get(3).copied().unwrap_or("0"));
        return (format!("常见问题 {number}"), format!("FAQ {number}"));
    }
    if path.contains("/qr_widget/") || path.ends_with("/media_slot") {
        return ("二维码".into(), "QR code".into());
    }
    if path.contains("/seo_") {
        return ("搜索主题区".into(), "Search topics".into());
    }
    if path.contains("/final_") {
        return ("页尾行动区".into(), "Final call to action".into());
    }
    if path.contains("/actions/") {
        return ("首屏按钮".into(), "Hero buttons".into());
    }
    if segments.get(1) == Some(&"content") {
        ("首页总体".into(), "Home page".into())
    } else {
        ("公共页头与页脚".into(), "Shared header and footer".into())
    }
}

fn field_label(path: &str) -> (String, String) {
    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let key = segments.last().copied().unwrap_or("value");
    let translated = match key {
        "label" => ("文字", "Label"),
        "href" => ("链接", "Link"),
        "title" => ("标题", "Title"),
        "heading" => ("标题", "Heading"),
        "description" => ("说明", "Description"),
        "lead" => ("引导文字", "Lead"),
        "body" => ("正文", "Body"),
        "meta" => ("补充信息", "Supporting text"),
        "question" => ("问题", "Question"),
        "answer" => ("回答", "Answer"),
        "planned" => ("规划中", "Planned"),
        "media_slot" => ("二维码", "QR code"),
        "footer_signature" => ("页脚签名", "Footer signature"),
        other => (other, other),
    };
    (translated.0.to_owned(), translated.1.to_owned())
}

fn number_label(value: &str) -> usize {
    value.parse::<usize>().unwrap_or(0) + 1
}
