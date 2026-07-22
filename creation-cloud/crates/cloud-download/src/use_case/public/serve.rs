//! 生成外链重定向或本站 Range 流式响应，并记录最小下载事件。

use std::io::SeekFrom;

use axum::{
    body::Body,
    http::{
        HeaderValue, Response, StatusCode,
        header::{
            ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG,
        },
    },
    response::{IntoResponse, Redirect},
};
use cloud_domain::AuthenticatedSession;
use cloud_domain::{AppError, AppResult};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncSeekExt},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{Service, SourceKind, local_file, range::ByteRange, repository, validation};

impl Service {
    pub async fn serve_download(
        &self,
        asset_id: Uuid,
        source_id: Uuid,
        range_header: Option<&str>,
    ) -> AppResult<Response<Body>> {
        self.serve_download_for_account(None, asset_id, source_id, range_header)
            .await
    }

    pub async fn serve_account_download(
        &self,
        session: &AuthenticatedSession,
        asset_id: Uuid,
        source_id: Uuid,
        range_header: Option<&str>,
    ) -> AppResult<Response<Body>> {
        self.serve_download_for_account(Some(session.account_id), asset_id, source_id, range_header)
            .await
    }

    async fn serve_download_for_account(
        &self,
        account_id: Option<Uuid>,
        asset_id: Uuid,
        source_id: Uuid,
        range_header: Option<&str>,
    ) -> AppResult<Response<Body>> {
        let asset_id = validation::valid_id(asset_id, "资产标识")?;
        let source_id = validation::valid_id(source_id, "来源标识")?;
        let _permit = self.limiter.acquire(source_id)?;
        let target = repository::public::target::execute(&self.pool, asset_id, source_id).await?;

        let response = match SourceKind::try_from(target.source_kind.as_str())? {
            SourceKind::External => external_response(
                target
                    .external_url
                    .as_deref()
                    .ok_or_else(|| AppError::Internal("外部来源缺少 URL".into()))?,
            )?,
            SourceKind::Local => {
                let relative = target
                    .local_path
                    .as_deref()
                    .ok_or_else(|| AppError::Internal("本站来源缺少相对路径".into()))?;
                local_response(
                    &self.file_verifier,
                    self.download_root.as_path(),
                    relative,
                    &target.file_name,
                    target.byte_size,
                    &target.sha256,
                    range_header,
                )
                .await?
            }
        };
        repository::public::record_event::execute(
            &self.pool,
            target.asset_id,
            target.source_id,
            account_id,
        )
        .await?;
        Ok(response)
    }
}

fn external_response(value: &str) -> AppResult<Response<Body>> {
    let url = validation::external_url(value)?;
    Ok(Redirect::temporary(&url).into_response())
}

async fn local_response(
    verifier: &crate::file_verification::FileVerifier,
    download_root: &std::path::Path,
    relative: &str,
    file_name: &str,
    expected_size: i64,
    sha256: &str,
    range_header: Option<&str>,
) -> AppResult<Response<Body>> {
    let path = local_file::resolve(download_root, relative).await?;
    let mut file = fs::File::open(&path)
        .await
        .map_err(|_| AppError::NotFound("本站下载文件不可读".into()))?;
    let actual_size = file
        .metadata()
        .await
        .map_err(|_| AppError::Internal("无法读取本站文件元数据".into()))?
        .len();
    if u64::try_from(expected_size).ok() != Some(actual_size) {
        return Err(AppError::Conflict("本站文件大小与发布资产不一致".into()));
    }
    let byte_range = match ByteRange::parse(range_header, actual_size) {
        Ok(value) => value,
        Err(()) => return range_not_satisfiable(actual_size),
    };
    verifier
        .verify(&path, &mut file, actual_size, sha256)
        .await?;
    let (start, end, partial) = byte_range.bounds(actual_size);
    let content_length = if partial {
        end - start + 1
    } else {
        actual_size
    };
    if start > 0 {
        file.seek(SeekFrom::Start(start))
            .await
            .map_err(|_| AppError::Internal("定位下载区间失败".into()))?;
    }
    let stream = ReaderStream::new(file.take(content_length));
    let status = if partial {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    let mut builder = Response::builder()
        .status(status)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, content_length.to_string())
        .header(
            CONTENT_TYPE,
            mime_guess::from_path(file_name)
                .first_or_octet_stream()
                .to_string(),
        )
        .header(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", safe_file_name(file_name)),
        )
        .header(ETAG, format!("\"{sha256}\""));
    if partial {
        builder = builder.header(CONTENT_RANGE, format!("bytes {start}-{end}/{actual_size}"));
    }
    builder
        .body(Body::from_stream(stream))
        .map_err(|_| AppError::Internal("构造下载响应失败".into()))
}

fn range_not_satisfiable(size: u64) -> AppResult<Response<Body>> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{size}"))
        .header(ACCEPT_RANGES, HeaderValue::from_static("bytes"))
        .body(Body::empty())
        .map_err(|_| AppError::Internal("构造 Range 错误响应失败".into()))
}

fn safe_file_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
