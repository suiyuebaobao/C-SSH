//! 接收独立前端的极简下载表单；与历史表单入口共享同一流式编排。
use crate::PreparedLocalUpload;
use axum::{
    Extension, Json, Router,
    extract::{DefaultBodyLimit, Multipart, State, multipart::Field},
    routing::post,
};
use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession};
use cloud_release::{CreateAssetInput, ReleaseAsset};
use uuid::Uuid;

#[derive(Clone)]
pub struct NewDownloadState {
    download: crate::Service,
    release: cloud_release::Service,
}
impl NewDownloadState {
    pub const fn new(download: crate::Service, release: cloud_release::Service) -> Self {
        Self { download, release }
    }
    const fn download(&self) -> &crate::Service {
        &self.download
    }
    const fn release(&self) -> &cloud_release::Service {
        &self.release
    }
}
#[must_use = "须挂载在管理员CSRF及审计链内"]
pub fn router(state: NewDownloadState) -> Router {
    Router::new()
        .route("/new", post(handle).layer(DefaultBodyLimit::disable()))
        .with_state(state)
}
async fn handle(
    State(state): State<NewDownloadState>,
    Extension(session): Extension<AuthenticatedSession>,
    multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let actor = AdminActor::from_session(&session)?;
    let (release_id, _) = submit(&state, &actor, multipart).await?;
    Ok(Json(serde_json::json!({ "release_id": release_id })))
}
pub async fn submit(
    state: &NewDownloadState,
    actor: &AdminActor,
    multipart: Multipart,
) -> AppResult<(Uuid, Option<String>)> {
    let form = parse_form(state, actor, multipart).await?;
    let locale = form.lang.clone();
    Ok((create_download(state, actor, form).await?, locale))
}
const MAX_TEXT_FIELD_BYTES: usize = 8 * 1024;

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
    updater_signature: Option<String>,
    lang: Option<String>,
    local_upload: Option<PreparedLocalUpload>,
}

async fn parse_form(
    state: &NewDownloadState,
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
            "updater_signature"
                if field
                    .file_name()
                    .is_some_and(|value| !value.trim().is_empty()) =>
            {
                set_once(
                    &mut form.updater_signature,
                    read_text(&mut field).await?,
                    "updater signature",
                )?;
            }
            "updater_signature" => {}
            "lang" => set_once(&mut form.lang, read_text(&mut field).await?, "语言")?,
            _ => return Err(AppError::Validation("新增下载表单包含未知字段".into())),
        }
    }
    Ok(form)
}

async fn create_download(
    state: &NewDownloadState,
    actor: &cloud_domain::AdminActor,
    mut form: NewDownloadForm,
) -> AppResult<Uuid> {
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
                    form.updater_signature.as_deref(),
                )
                .await?;
            Ok(release_id)
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
                return Ok(release_id);
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
            Ok(release_id)
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
