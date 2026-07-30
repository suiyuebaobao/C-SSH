//! 管理员读取和以 revision CAS 更新普通用户邮箱验证码全局开关。

use chrono::{DateTime, Utc};
use cloud_domain::{AdminActor, AppError, AppResult, mark_semantic_audit_recorded};
use cloud_store::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository;

#[derive(Clone, Debug, Serialize)]
pub struct AuthSettings {
    pub email_verification_enabled: bool,
    pub revision: i64,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateAuthSettings {
    pub email_verification_enabled: bool,
    pub expected_revision: i64,
}

pub(crate) async fn get(pool: &PgPool, actor: &AdminActor) -> AppResult<AuthSettings> {
    require_actor(actor)?;
    repository::settings::read(pool).await
}

pub(crate) async fn update(
    pool: &PgPool,
    actor: &AdminActor,
    input: UpdateAuthSettings,
) -> AppResult<AuthSettings> {
    let actor_id = require_actor(actor)?;
    if input.expected_revision < 1 {
        return Err(AppError::Validation(
            "expected_revision 必须大于零".to_owned(),
        ));
    }
    let mut transaction = pool
        .begin()
        .await
        .map_err(|_| AppError::Storage("认证设置事务启动失败".to_owned()))?;
    let current = repository::settings::lock(&mut transaction).await?;
    if current.revision != input.expected_revision {
        return Err(AppError::Conflict(
            "认证设置已变化，请刷新后重试".to_owned(),
        ));
    }
    if current.email_verification_enabled == input.email_verification_enabled {
        return Err(AppError::Conflict("认证设置没有变化".to_owned()));
    }
    let updated = repository::settings::update(
        &mut transaction,
        actor_id,
        input.expected_revision,
        input.email_verification_enabled,
    )
    .await?;
    let invalidated = if updated.email_verification_enabled {
        (0, 0)
    } else {
        repository::settings::invalidate_open_challenges(&mut transaction).await?
    };
    repository::settings::audit(&mut transaction, actor_id, &updated, invalidated).await?;
    transaction
        .commit()
        .await
        .map_err(|_| AppError::Storage("认证设置事务提交失败".to_owned()))?;
    mark_semantic_audit_recorded();
    Ok(updated)
}

fn require_actor(actor: &AdminActor) -> AppResult<Uuid> {
    let actor_id = actor.account_id();
    if actor_id.is_nil() {
        Err(AppError::Unauthorized("管理员身份无效".to_owned()))
    } else {
        Ok(actor_id)
    }
}
