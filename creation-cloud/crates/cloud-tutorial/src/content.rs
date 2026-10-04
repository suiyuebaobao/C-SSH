//! 按发布状态分发单个教学文件，复用共享单段Range解析并流式响应。

use axum::{
    body::Body,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use cloud_domain::{AppError, AppResult, ByteRange};
use std::io::SeekFrom;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{Service, repository, storage, validation};

pub(crate) async fn response(
    service: &Service,
    id: Uuid,
    poster: bool,
    public: bool,
    headers: HeaderMap,
) -> AppResult<Response> {
    validation::id(id)?;
    let row = repository::get(&service.pool, id).await?;
    if row.state == "deleted" || (public && row.state != "published") {
        return Err(AppError::NotFound("教学视频未公开".into()));
    }
    let (size, sha, content_type) = if poster {
        (row.poster_bytes as u64, row.poster_sha256, "image/png")
    } else {
        (row.video_bytes as u64, row.video_sha256, "video/mp4")
    };
    let etag = format!("\"{sha}\"");
    let range_header = if headers
        .get(header::IF_RANGE)
        .is_some_and(|value| value.as_bytes() != etag.as_bytes())
    {
        None
    } else {
        headers.get(header::RANGE)
    };
    let range = range_header
        .map(|value| value.to_str())
        .transpose()
        .ok()
        .and_then(|value| ByteRange::parse(value, size).ok());
    let Some(range) = range else {
        return Ok((
            StatusCode::RANGE_NOT_SATISFIABLE,
            [
                (header::CONTENT_RANGE, format!("bytes */{size}")),
                (header::CACHE_CONTROL, "no-store".into()),
            ],
        )
            .into_response());
    };
    let (start, end, partial) = range.bounds(size);
    let length = end - start + 1;
    let root = storage::prepare(&service.media_base).await?;
    let mut file = storage::open(&root, id, poster, size).await?;
    file.seek(SeekFrom::Start(start))
        .await
        .map_err(storage::io_error)?;
    let mut response = Response::builder()
        .status(if partial {
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        })
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, etag)
        .header(header::CACHE_CONTROL, "no-store")
        .header("x-content-type-options", "nosniff");
    if partial {
        response = response.header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}"));
    }
    response
        .body(Body::from_stream(ReaderStream::new(file.take(length))))
        .map_err(|_| AppError::Internal("无法生成教学媒体响应".into()))
}
