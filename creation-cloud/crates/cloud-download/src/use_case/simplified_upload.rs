//! 为极简下载表单提供组合用例：服务端计算本站文件身份，并在同一事务创建资产与来源。

use axum::extract::multipart::Field;
use cloud_domain::{AdminActor, AppResult};
use cloud_release::CreateAssetInput;
use url::Url;
use uuid::Uuid;

use crate::{
    CreateSourceInput, PreparedLocalUpload, ReleaseSource, Service, SourceKind, authorization,
    repository, upload_file, validation,
};

impl Service {
    pub async fn prepare_local_asset(
        &self,
        actor: &AdminActor,
        field: &mut Field<'_>,
    ) -> AppResult<PreparedLocalUpload> {
        authorization::require(actor)?;
        upload_file::stage(self.download_root.as_path(), field).await
    }

    pub async fn create_local_download(
        &self,
        actor: &AdminActor,
        mut asset_input: CreateAssetInput,
        mut prepared: PreparedLocalUpload,
        updater_signature: Option<&str>,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        let updater_signature = crate::signature::validate_for_asset(
            &asset_input.platform,
            &asset_input.package_kind,
            updater_signature,
        )?;
        asset_input.file_name = prepared.file_name().to_owned();
        asset_input.byte_size = prepared.byte_size();
        asset_input.sha256 = prepared.sha256().to_owned();
        let relative_path = prepared.promote().await?;

        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let asset =
            cloud_release::create_asset_in_transaction(actor, &mut transaction, asset_input)
                .await?;
        if let Some(installed_sha256) = automatic_installed_sha256(&asset) {
            cloud_release::record_installed_sha256_in_transaction(
                actor,
                &mut transaction,
                asset.id,
                installed_sha256,
            )
            .await?;
        }
        if let Some(signature) = updater_signature.as_deref() {
            crate::signature::set_in_transaction(&mut transaction, asset.id, signature).await?;
        }
        let source_input = CreateSourceInput {
            asset_id: asset.id,
            source_kind: SourceKind::Local,
            provider_name: "本站".to_owned(),
            local_path: Some(validation::local_path(&relative_path)?),
            external_url: None,
            sort_order: 0,
            enabled: true,
        };
        let source =
            repository::source::create::execute_in_transaction(&mut transaction, &source_input)
                .await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        prepared.disarm();
        Ok(source)
    }

    pub async fn create_external_download(
        &self,
        actor: &AdminActor,
        asset_input: CreateAssetInput,
        external_url: &str,
    ) -> AppResult<ReleaseSource> {
        authorization::require(actor)?;
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(repository::map_transaction_error)?;
        let asset =
            cloud_release::create_asset_in_transaction(actor, &mut transaction, asset_input)
                .await?;
        let source_input = external_source_input(asset.id, external_url)?;
        let source =
            repository::source::create::execute_in_transaction(&mut transaction, &source_input)
                .await?;
        transaction
            .commit()
            .await
            .map_err(repository::map_transaction_error)?;
        Ok(source)
    }

    pub async fn create_external_source(
        &self,
        actor: &AdminActor,
        asset_id: Uuid,
        external_url: &str,
    ) -> AppResult<ReleaseSource> {
        self.create_source(actor, external_source_input(asset_id, external_url)?)
            .await
    }
}

fn automatic_installed_sha256(asset: &cloud_release::ReleaseAsset) -> Option<&str> {
    matches!(
        (
            asset.platform.as_str(),
            asset.architecture.as_str(),
            asset.package_kind.as_str(),
        ),
        ("android", "aarch64", "apk")
    )
    .then_some(asset.sha256.as_str())
}

fn external_source_input(asset_id: Uuid, external_url: &str) -> AppResult<CreateSourceInput> {
    let external_url = validation::external_url(external_url)?;
    let parsed = Url::parse(&external_url)
        .map_err(|_| cloud_domain::AppError::Validation("外部来源 URL 格式无效".into()))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| cloud_domain::AppError::Validation("外部来源 URL 缺少主机".into()))?
        .to_ascii_lowercase();
    let provider_name = if host == "github.com" || host.ends_with(".github.com") {
        "GitHub Release".to_owned()
    } else {
        host.chars().take(100).collect()
    };
    Ok(CreateSourceInput {
        asset_id,
        source_kind: SourceKind::External,
        provider_name,
        local_path: None,
        external_url: Some(external_url),
        sort_order: 0,
        enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn derives_external_provider_without_accepting_metadata_from_the_form() {
        let source = external_source_input(
            Uuid::now_v7(),
            "https://github.com/example/project/releases/download/v7.0.0/client.exe",
        )
        .expect("GitHub Release 地址应合法");
        assert_eq!(source.provider_name, "GitHub Release");
        assert_eq!(source.source_kind, SourceKind::External);
        assert!(source.local_path.is_none());
    }

    #[test]
    fn external_source_keeps_existing_https_validation() {
        assert!(external_source_input(Uuid::now_v7(), "http://example.com/client.exe").is_err());
        assert!(
            external_source_input(Uuid::now_v7(), "https://example.com/client.exe?token=x")
                .is_err()
        );
    }

    #[test]
    fn only_android_apk_can_derive_installed_identity_from_uploaded_asset() {
        let mut asset = cloud_release::ReleaseAsset {
            id: Uuid::now_v7(),
            release_id: Uuid::now_v7(),
            platform: "android".into(),
            architecture: "aarch64".into(),
            package_kind: "apk".into(),
            file_name: "client.apk".into(),
            byte_size: 1,
            sha256: "a".repeat(64),
            installed_sha256: None,
            created_at: Utc::now(),
        };
        assert_eq!(
            automatic_installed_sha256(&asset),
            Some(asset.sha256.as_str())
        );
        asset.platform = "windows".into();
        asset.package_kind = "exe".into();
        assert_eq!(automatic_installed_sha256(&asset), None);
    }
}
