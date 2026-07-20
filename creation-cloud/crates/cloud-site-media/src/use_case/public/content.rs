//! 仅为仍处于发布状态的记录读取并复核受控 PNG 文件身份。

use cloud_domain::AppResult;
use uuid::Uuid;

use crate::{Service, model::PublicMediaContent, repository, storage, validation};

impl Service {
    pub(crate) async fn content(&self, media_id: Uuid) -> AppResult<PublicMediaContent> {
        let media =
            repository::public::content::execute(self.pool(), validation::valid_id(media_id)?)
                .await?;
        let bytes = storage::read_verified(
            self.site_media_root(),
            &media.storage_key,
            media.byte_size,
            &media.sha256,
        )
        .await?;
        Ok(PublicMediaContent {
            bytes,
            sha256: media.sha256,
        })
    }
}
