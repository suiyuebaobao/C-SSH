//! 把极简“新增下载”表单编排到既有资产与下载来源强校验用例。
//! 本地文件身份由服务端计算；外链仅在没有匹配资产时读取折叠校验字段。

use axum::{
    Extension,
    extract::{Multipart, State, multipart::Field},
    http::HeaderMap,
    response::Response,
};
use cloud_domain::{AppError, AppResult, AuthenticatedSession};
use cloud_download::PreparedLocalUpload;
use cloud_release::{CreateAssetInput, ReleaseAsset};
use cloud_site::Locale;
use uuid::Uuid;

use crate::AdminPageState;

use super::super::shared;

const MAX_TEXT_FIELD_BYTES: usize = 2 * 1024;

#[derive(Default)]
struct NewDownloadForm {
    release_id: Option<String>,
    platform: Option<String>,
    architecture: Option<String>,
    package_kind: Option<String>,
    source_mode: Option<String>,
    external_url: Option<String>,
    file_name: Option<String>,
    byte_size: Option<String>,
    sha256: Option<String>,
    lang: Option<String>,
    local_upload: Option<PreparedLocalUpload>,
}

pub(crate) async fn handle(
    State(state): State<AdminPageState>,
    Extension(session): Extension<AuthenticatedSession>,
    headers: HeaderMap,
    multipart: Multipart,
) -> Response {
    let actor = match shared::actor_from_session(&session) {
        Ok(actor) => actor,
        Err(error) => return shared::action_error(Locale::ZhCn, error),
    };
    let form = match parse_form(&state, &actor, multipart).await {
        Ok(form) => form,
        Err(error) => return shared::action_error(Locale::ZhCn, error),
    };
    let locale = shared::locale(form.lang.as_deref());
    match create_download(&state, &actor, form).await {
        Ok(()) => shared::action_success(&headers, "/admin/assets", locale),
        Err(error) => shared::action_error(locale, error),
    }
}

async fn parse_form(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
    mut multipart: Multipart,
) -> AppResult<NewDownloadForm> {
    let mut form = NewDownloadForm::default();
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::Validation("新增下载表单格式无效".into()))?
    {
        let name = field
            .name()
            .map(str::to_owned)
            .ok_or_else(|| AppError::Validation("新增下载字段缺少名称".into()))?;
        match name.as_str() {
            "file"
                if field
                    .file_name()
                    .is_some_and(|value| !value.trim().is_empty()) =>
            {
                if form.local_upload.is_some() {
                    return Err(AppError::Validation("只能选择一个本地文件".into()));
                }
                form.local_upload = Some(
                    state
                        .download()
                        .prepare_local_asset(actor, &mut field)
                        .await?,
                );
            }
            "file" => {}
            "release_id" => set_once(
                &mut form.release_id,
                read_text(&mut field).await?,
                "发布版本",
            )?,
            "platform" => set_once(&mut form.platform, read_text(&mut field).await?, "平台")?,
            "architecture" => {
                set_once(&mut form.architecture, read_text(&mut field).await?, "架构")?;
            }
            "package_kind" => {
                set_once(
                    &mut form.package_kind,
                    read_text(&mut field).await?,
                    "包类型",
                )?;
            }
            "source_mode" => {
                set_once(
                    &mut form.source_mode,
                    read_text(&mut field).await?,
                    "下载方式",
                )?;
            }
            "external_url" => {
                set_once(
                    &mut form.external_url,
                    read_text(&mut field).await?,
                    "外链地址",
                )?;
            }
            "file_name" => {
                set_once(&mut form.file_name, read_text(&mut field).await?, "文件名")?;
            }
            "byte_size" => {
                set_once(&mut form.byte_size, read_text(&mut field).await?, "字节数")?;
            }
            "sha256" => {
                set_once(&mut form.sha256, read_text(&mut field).await?, "SHA256")?;
            }
            "lang" => set_once(&mut form.lang, read_text(&mut field).await?, "语言")?,
            _ => return Err(AppError::Validation("新增下载表单包含未知字段".into())),
        }
    }
    Ok(form)
}

async fn create_download(
    state: &AdminPageState,
    actor: &cloud_domain::AdminActor,
    mut form: NewDownloadForm,
) -> AppResult<()> {
    let release_id = required(form.release_id.take(), "发布版本")?
        .parse::<Uuid>()
        .map_err(|_| AppError::Validation("发布版本无效".into()))?;
    let platform = required(form.platform.take(), "平台")?;
    let architecture = required(form.architecture.take(), "架构")?;
    let package_kind = required(form.package_kind.take(), "包类型")?;
    let mode = required(form.source_mode.take(), "下载方式")?;

    match mode.as_str() {
        "local" => {
            if form
                .external_url
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                return Err(AppError::Validation("本地上传不能同时填写外链地址".into()));
            }
            let prepared = form
                .local_upload
                .take()
                .ok_or_else(|| AppError::Validation("请选择本地文件".into()))?;
            state
                .download()
                .create_local_download(
                    actor,
                    CreateAssetInput {
                        release_id,
                        platform,
                        architecture,
                        package_kind,
                        file_name: prepared.file_name().to_owned(),
                        byte_size: prepared.byte_size(),
                        sha256: prepared.sha256().to_owned(),
                    },
                    prepared,
                )
                .await?;
            Ok(())
        }
        "external" => {
            if form.local_upload.is_some() {
                return Err(AppError::Validation("外链下载不能同时上传本地文件".into()));
            }
            let external_url = required(form.external_url.take(), "HTTPS URL")?;
            let existing = state
                .release()
                .list_assets(actor, release_id)
                .await?
                .into_iter()
                .find(|asset| matches_asset(asset, &platform, &architecture, &package_kind));
            if let Some(asset) = existing {
                state
                    .download()
                    .create_external_source(actor, asset.id, &external_url)
                    .await?;
                return Ok(());
            }

            let file_name = required(form.file_name.take(), "文件名")?;
            let byte_size = required(form.byte_size.take(), "字节数")?
                .parse::<i64>()
                .map_err(|_| AppError::Validation("字节数必须是整数".into()))?;
            let sha256 = required(form.sha256.take(), "SHA256")?;
            state
                .download()
                .create_external_download(
                    actor,
                    CreateAssetInput {
                        release_id,
                        platform,
                        architecture,
                        package_kind,
                        file_name,
                        byte_size,
                        sha256,
                    },
                    &external_url,
                )
                .await?;
            Ok(())
        }
        _ => Err(AppError::Validation(
            "下载方式只允许本地上传或外链下载".into(),
        )),
    }
}

fn matches_asset(
    asset: &ReleaseAsset,
    platform: &str,
    architecture: &str,
    package_kind: &str,
) -> bool {
    asset.platform.eq_ignore_ascii_case(platform.trim())
        && asset.architecture.eq_ignore_ascii_case(architecture.trim())
        && asset.package_kind.eq_ignore_ascii_case(package_kind.trim())
}

async fn read_text(field: &mut Field<'_>) -> AppResult<String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| AppError::Validation("读取表单字段失败".into()))?
    {
        if bytes.len().saturating_add(chunk.len()) > MAX_TEXT_FIELD_BYTES {
            return Err(AppError::Validation("表单字段过长".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| AppError::Validation("表单字段必须是 UTF-8".into()))
}

fn set_once(target: &mut Option<String>, value: String, field: &str) -> AppResult<()> {
    if target.replace(value).is_some() {
        return Err(AppError::Validation(format!("{field}不能重复")));
    }
    Ok(())
}

fn required(value: Option<String>, field: &str) -> AppResult<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Validation(format!("{field}不能为空")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_exactly_one_download_mode() {
        assert!(required(Some(" local ".to_owned()), "下载方式").is_ok());
        assert!(required(Some("".to_owned()), "下载方式").is_err());
        assert!(required(None, "下载方式").is_err());
    }

    #[test]
    fn duplicate_simple_fields_are_rejected() {
        let mut value = None;
        set_once(&mut value, "windows".to_owned(), "平台").unwrap();
        assert!(set_once(&mut value, "linux".to_owned(), "平台").is_err());
    }

    #[test]
    fn existing_asset_match_uses_the_release_identity_tuple() {
        let asset = ReleaseAsset {
            id: Uuid::now_v7(),
            release_id: Uuid::now_v7(),
            platform: "windows".to_owned(),
            architecture: "x86_64".to_owned(),
            package_kind: "exe".to_owned(),
            file_name: "client.exe".to_owned(),
            byte_size: 10,
            sha256: "a".repeat(64),
            created_at: chrono::Utc::now(),
        };
        assert!(matches_asset(&asset, " Windows ", "X86_64", "EXE"));
        assert!(!matches_asset(&asset, "linux", "x86_64", "exe"));
    }
}
