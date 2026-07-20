//! 对上传图片做格式探测、有界解码和去元数据 PNG 重编码。
//! 本模块只验证像素与文件身份，绝不解释或记录二维码载荷。

use std::io::Cursor;

use cloud_domain::{AppError, AppResult};
use image::{
    ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, Limits, codecs::png::PngEncoder,
};
use sha2::{Digest, Sha256};

pub(crate) const MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const MIN_DIMENSION: u32 = 128;
pub(crate) const MAX_DIMENSION: u32 = 2048;
const MAX_DECODE_ALLOC: u64 = 32 * 1024 * 1024;

pub(crate) struct ValidatedImage {
    pub png: Vec<u8>,
    pub sha256: String,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn validate_and_reencode(
    declared_content_type: &str,
    bytes: &[u8],
) -> AppResult<ValidatedImage> {
    if bytes.is_empty() || bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::Validation(
            "图片大小必须在 1 字节到 2 MiB 之间".into(),
        ));
    }

    let format = image::guess_format(bytes)
        .map_err(|_| AppError::Validation("文件不是可识别的 PNG 或 JPEG 图片".into()))?;
    ensure_supported_format(declared_content_type, format)?;

    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| AppError::Validation("图片损坏、尺寸超限或无法安全解码".into()))?;
    let width = image.width();
    let height = image.height();
    if width != height || !(MIN_DIMENSION..=MAX_DIMENSION).contains(&width) {
        return Err(AppError::Validation(
            "图片必须为 128 到 2048 像素的正方形".into(),
        ));
    }

    let rgba = image.to_rgba8();
    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(rgba.as_raw(), width, height, ExtendedColorType::Rgba8)
        .map_err(|_| AppError::Internal("站点媒体 PNG 重编码失败".into()))?;
    if png.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::Validation("重编码后的 PNG 超过 2 MiB".into()));
    }
    let sha256 = hex::encode(Sha256::digest(&png));
    Ok(ValidatedImage {
        png,
        sha256,
        width,
        height,
    })
}

fn ensure_supported_format(declared: &str, detected: ImageFormat) -> AppResult<()> {
    let expected = match detected {
        ImageFormat::Png => "image/png",
        ImageFormat::Jpeg => "image/jpeg",
        _ => {
            return Err(AppError::Validation("只允许上传 PNG 或 JPEG 图片".into()));
        }
    };
    if declared.trim().to_ascii_lowercase() != expected {
        return Err(AppError::Validation("图片 MIME 与真实格式不一致".into()));
    }
    Ok(())
}
