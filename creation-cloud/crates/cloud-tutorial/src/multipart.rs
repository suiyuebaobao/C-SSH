//! 流式接收有界MP4和封面；只对封面解码，取消时删除本次临时文件。

use axum::extract::{Multipart, multipart::Field};
use cloud_domain::{AppError, AppResult};
use image::{ImageFormat, ImageReader};
use sha2::{Digest, Sha256};
use std::{io::Cursor, path::Path};
use tokio::{fs, io::AsyncWriteExt};

use crate::{
    model::Metadata,
    storage::{self, StagedAsset},
    validation,
};

pub(crate) struct Upload {
    pub metadata: Metadata,
    pub video: StagedAsset,
    pub poster: StagedAsset,
}

pub(crate) async fn receive(root: &Path, mut multipart: Multipart) -> AppResult<Upload> {
    let (mut metadata, mut video, mut poster) = (None, None, None);
    while let Some(mut field) = multipart.next_field().await.map_err(|_| invalid())? {
        match field.name() {
            Some("metadata") if metadata.is_none() => {
                let bytes = bounded(&mut field, 24576).await?;
                let input = serde_json::from_slice::<Metadata>(&bytes).map_err(|_| invalid())?;
                metadata = Some(validation::metadata(input)?);
            }
            Some("video") if video.is_none() => video = Some(video_file(root, &mut field).await?),
            Some("poster") if poster.is_none() => {
                let bytes = bounded(&mut field, validation::MAX_POSTER_BYTES).await?;
                let normalized = poster_png(&bytes)?;
                poster = Some(storage::stage_bytes(root, &normalized).await?);
            }
            _ => return Err(AppError::Validation("教学上传包含未知或重复字段".into())),
        }
    }
    Ok(Upload {
        metadata: metadata.ok_or_else(invalid)?,
        video: video.ok_or_else(invalid)?,
        poster: poster.ok_or_else(invalid)?,
    })
}

async fn video_file(root: &Path, field: &mut Field<'_>) -> AppResult<StagedAsset> {
    if field.content_type() != Some("video/mp4") {
        return Err(AppError::Validation("教学影片必须为MP4".into()));
    }
    let temp = storage::temporary(root)?;
    let mut file = fs::File::from_std(temp.reopen().map_err(storage::io_error)?);
    let mut size = 0_u64;
    let mut hasher = Sha256::new();
    let mut prefix = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(|_| invalid())? {
        size = size.checked_add(chunk.len() as u64).ok_or_else(invalid)?;
        if size > validation::MAX_VIDEO_BYTES {
            return Err(AppError::Validation("教学影片超过200MiB".into()));
        }
        if prefix.len() < 64 {
            prefix.extend_from_slice(&chunk[..chunk.len().min(64 - prefix.len())]);
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(storage::io_error)?;
    }
    validate_mp4(&prefix, size)?;
    file.sync_all().await.map_err(storage::io_error)?;
    Ok(StagedAsset {
        temp,
        bytes: size,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

pub(crate) fn validate_mp4(prefix: &[u8], size: u64) -> AppResult<()> {
    if prefix.len() < 16 || size < 16 || &prefix[4..8] != b"ftyp" {
        return Err(AppError::Validation("影片不是有效的MP4容器".into()));
    }
    let box_size = u32::from_be_bytes(prefix[..4].try_into().map_err(|_| invalid())?) as u64;
    if box_size < 16 || box_size > size {
        return Err(AppError::Validation("MP4类型区块长度无效".into()));
    }
    let brands = [b"isom", b"iso2", b"mp41", b"mp42", b"avc1", b"M4V "];
    let end = prefix.len().min(box_size as usize);
    if !prefix[8..end]
        .chunks_exact(4)
        .any(|brand| brands.iter().any(|known| brand == *known))
    {
        return Err(AppError::Validation("影片容器不属于支持的MP4类型".into()));
    }
    Ok(())
}

pub(crate) fn poster_png(bytes: &[u8]) -> AppResult<Vec<u8>> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| invalid())?;
    let format = reader.format().ok_or_else(invalid)?;
    if !matches!(format, ImageFormat::Png | ImageFormat::Jpeg) {
        return Err(AppError::Validation("封面只支持PNG或JPEG".into()));
    }
    let (width, height) = reader.into_dimensions().map_err(|_| invalid())?;
    if !(16..=4096).contains(&width) || !(16..=4096).contains(&height) {
        return Err(AppError::Validation(
            "封面尺寸必须在16至4096像素之间".into(),
        ));
    }
    let image = image::load_from_memory_with_format(bytes, format).map_err(|_| invalid())?;
    let mut result = Cursor::new(Vec::new());
    image
        .write_to(&mut result, ImageFormat::Png)
        .map_err(|_| invalid())?;
    Ok(result.into_inner())
}

async fn bounded(field: &mut Field<'_>, maximum: usize) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.chunk().await.map_err(|_| invalid())? {
        if bytes.len().saturating_add(chunk.len()) > maximum {
            return Err(AppError::Validation("教学上传字段超过大小限制".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn invalid() -> AppError {
    AppError::Validation("教学上传字段缺失、损坏或格式无效".into())
}
