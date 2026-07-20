//! 将站点媒体 multipart 请求严格映射为单张图片和双语替代文本。

use axum::extract::Multipart;
use cloud_domain::{AppError, AppResult};

use crate::{CreateSiteMediaInput, image_validation::MAX_UPLOAD_BYTES};

pub(crate) async fn parse(mut multipart: Multipart) -> AppResult<CreateSiteMediaInput> {
    let mut bytes = None;
    let mut declared_content_type = None;
    let mut alt_zh = None;
    let mut alt_en = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("multipart 上传格式无效".into()))?
    {
        let name = field
            .name()
            .ok_or_else(|| AppError::Validation("multipart 字段缺少名称".into()))?
            .to_owned();
        match name.as_str() {
            "file" => {
                if bytes.is_some() {
                    return Err(AppError::Validation("只能上传一个图片文件".into()));
                }
                let content_type = field
                    .content_type()
                    .ok_or_else(|| AppError::Validation("图片缺少 Content-Type".into()))?
                    .to_owned();
                let data = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::Validation("读取上传图片失败".into()))?;
                if data.len() > MAX_UPLOAD_BYTES {
                    return Err(AppError::Validation("上传图片不能超过 2 MiB".into()));
                }
                bytes = Some(data.to_vec());
                declared_content_type = Some(content_type);
            }
            "alt_zh" => set_text(&mut alt_zh, field.text().await, "中文替代文本")?,
            "alt_en" => set_text(&mut alt_en, field.text().await, "英文替代文本")?,
            _ => return Err(AppError::Validation("multipart 包含未知字段".into())),
        }
    }

    Ok(CreateSiteMediaInput {
        declared_content_type: declared_content_type
            .ok_or_else(|| AppError::Validation("缺少图片文件".into()))?,
        bytes: bytes.ok_or_else(|| AppError::Validation("缺少图片文件".into()))?,
        alt_zh: alt_zh.ok_or_else(|| AppError::Validation("缺少中文替代文本".into()))?,
        alt_en: alt_en.ok_or_else(|| AppError::Validation("缺少英文替代文本".into()))?,
    })
}

fn set_text(
    target: &mut Option<String>,
    value: Result<String, axum::extract::multipart::MultipartError>,
    label: &str,
) -> AppResult<()> {
    if target.is_some() {
        return Err(AppError::Validation(format!("{label}不能重复")));
    }
    *target = Some(value.map_err(|_| AppError::Validation(format!("读取{label}失败")))?);
    Ok(())
}
