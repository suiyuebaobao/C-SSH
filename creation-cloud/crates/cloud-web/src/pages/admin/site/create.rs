//! 解析受限内存中的首页二维码上传并调用站点媒体创建用例。
//! 文件最多 2 MiB；内容类型、正方形尺寸与重编码由领域 Service 继续校验。

use axum::{
    Extension,
    extract::{Multipart, Query, State},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_site_media::CreateSiteMediaInput;
use serde::Deserialize;

use crate::AdminPageState;

use super::super::shared;

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_ALT_BYTES: usize = 1_024;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct CreateQuery {
    lang: Option<String>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    Query(query): Query<CreateQuery>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let locale = shared::locale(query.lang.as_deref());
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(locale, error),
    };
    let input = match parse(multipart).await {
        Ok(input) => input,
        Err(error) => return shared::action_error(locale, error),
    };
    match state.site_media().create(&actor, input).await {
        Ok(_) => shared::action_success(&headers, "/admin/site", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

async fn parse(mut multipart: Multipart) -> AppResult<CreateSiteMediaInput> {
    let mut bytes = None;
    let mut content_type = None;
    let mut alt_zh = None;
    let mut alt_en = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("multipart 上传格式无效".to_owned()))?
    {
        let name = field
            .name()
            .ok_or_else(|| AppError::Validation("multipart 字段缺少名称".to_owned()))?
            .to_owned();
        match name.as_str() {
            "file" if bytes.is_none() => {
                content_type = Some(
                    field
                        .content_type()
                        .ok_or_else(|| AppError::Validation("图片缺少 Content-Type".to_owned()))?
                        .to_owned(),
                );
                let mut data = Vec::new();
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|_| AppError::Validation("读取上传图片失败".to_owned()))?
                {
                    if data.len().saturating_add(chunk.len()) > MAX_IMAGE_BYTES {
                        return Err(AppError::Validation("上传图片不能超过 2 MiB".to_owned()));
                    }
                    data.extend_from_slice(&chunk);
                }
                bytes = Some(data);
            }
            "file" => return Err(AppError::Validation("只能上传一个图片文件".to_owned())),
            "alt_zh" if alt_zh.is_none() => alt_zh = Some(read_text(field).await?),
            "alt_en" if alt_en.is_none() => alt_en = Some(read_text(field).await?),
            "alt_zh" | "alt_en" => {
                return Err(AppError::Validation("替代文本字段不能重复".to_owned()));
            }
            _ => return Err(AppError::Validation("multipart 包含未知字段".to_owned())),
        }
    }
    Ok(CreateSiteMediaInput {
        declared_content_type: content_type
            .ok_or_else(|| AppError::Validation("缺少图片文件".to_owned()))?,
        bytes: bytes.ok_or_else(|| AppError::Validation("缺少图片文件".to_owned()))?,
        alt_zh: alt_zh.ok_or_else(|| AppError::Validation("缺少中文替代文本".to_owned()))?,
        alt_en: alt_en.ok_or_else(|| AppError::Validation("缺少英文替代文本".to_owned()))?,
    })
}

async fn read_text(field: axum::extract::multipart::Field<'_>) -> AppResult<String> {
    let bytes = field
        .bytes()
        .await
        .map_err(|_| AppError::Validation("读取替代文本失败".to_owned()))?;
    if bytes.len() > MAX_ALT_BYTES {
        return Err(AppError::Validation("替代文本过长".to_owned()));
    }
    String::from_utf8(bytes.to_vec())
        .map_err(|_| AppError::Validation("替代文本必须是 UTF-8".to_owned()))
}
