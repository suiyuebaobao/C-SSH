//! 校验完整 mutation，并调用仓储执行幂等、冲突或原子写入。

use cloud_domain::{AppError, AppResult};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{PushOutcome, PushRequest, Service, repository, validation};

impl Service {
    pub async fn push(&self, account_id: Uuid, request: PushRequest) -> AppResult<PushOutcome> {
        validation::account(account_id)?;
        validation::push(&request)?;
        let fingerprint = fingerprint(&request)?;
        repository::push(&self.pool, account_id, &request, &fingerprint).await
    }
}

fn fingerprint(request: &PushRequest) -> AppResult<String> {
    let encoded = serde_json::to_vec(request)
        .map_err(|_| AppError::Internal("无法计算同步 mutation 指纹".to_owned()))?;
    Ok(hex::encode(Sha256::digest(encoded)))
}
