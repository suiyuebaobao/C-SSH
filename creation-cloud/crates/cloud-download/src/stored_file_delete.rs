//! 将服务端生成的本站对象安全移入隔离区，并在数据库结果明确后清理或恢复。
//! 本模块只接受数据库中的规范对象键，不接收浏览器提供的文件系统路径。

use std::{io::ErrorKind, path::PathBuf};

use cloud_domain::{AppError, AppResult};
use uuid::Uuid;

use crate::{local_file, upload_file::UploadLayout};

pub(crate) struct QuarantinedObject {
    original: PathBuf,
    quarantined: Option<PathBuf>,
}

impl QuarantinedObject {
    pub async fn isolate(download_root: &std::path::Path, relative: &str) -> AppResult<Self> {
        let object_id = parse_object_key(relative)?;
        let layout = UploadLayout::prepare(download_root).await?;
        let original = local_file::resolve(download_root, relative).await?;
        if original != layout.object_path(object_id) {
            return Err(AppError::Forbidden("本站来源不是服务端管理的对象".into()));
        }

        let quarantined = layout.deletion_path(Uuid::now_v7());
        match std::fs::symlink_metadata(&quarantined) {
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Ok(_) => return Err(AppError::Conflict("删除隔离目标已经存在".into())),
            Err(_) => return Err(AppError::Storage("无法检查删除隔离目标".into())),
        }
        std::fs::rename(&original, &quarantined)
            .map_err(|_| AppError::Storage("无法隔离本站来源文件".into()))?;
        // 同步 rename 后立即建立恢复守卫，中间不存在可取消的异步点。
        Ok(Self {
            original,
            quarantined: Some(quarantined),
        })
    }

    pub fn restore(&mut self) -> AppResult<()> {
        let Some(quarantined) = self.quarantined.as_ref() else {
            return Ok(());
        };
        match std::fs::symlink_metadata(&self.original) {
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(AppError::Conflict(
                    "本站来源原路径已被占用，拒绝覆盖".into(),
                ));
            }
            Err(_) => return Err(AppError::Storage("无法检查本站来源恢复目标".into())),
        }
        std::fs::rename(quarantined, &self.original)
            .map_err(|_| AppError::Storage("无法恢复本站来源文件".into()))?;
        self.quarantined = None;
        Ok(())
    }

    pub fn finish(mut self) -> AppResult<()> {
        let Some(quarantined) = self.quarantined.take() else {
            return Ok(());
        };
        // 数据库已经提交时不再恢复到公开对象目录；清理失败只保留不可下载的隔离文件。
        std::fs::remove_file(quarantined)
            .map_err(|_| AppError::Storage("无法清理已删除来源的隔离文件".into()))
    }
}

impl Drop for QuarantinedObject {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn parse_object_key(relative: &str) -> AppResult<Uuid> {
    let normalized = crate::validation::local_path(relative)?;
    let Some(object_name) = normalized.strip_prefix("objects/") else {
        return Err(AppError::Forbidden("本站来源不是服务端对象键".into()));
    };
    if object_name.contains('/') {
        return Err(AppError::Forbidden("本站来源对象键层级无效".into()));
    }
    let object_id = Uuid::parse_str(object_name)
        .map_err(|_| AppError::Forbidden("本站来源对象键格式无效".into()))?;
    if object_id.to_string() != object_name {
        return Err(AppError::Forbidden("本站来源对象键不是规范标识".into()));
    }
    Ok(object_id)
}
