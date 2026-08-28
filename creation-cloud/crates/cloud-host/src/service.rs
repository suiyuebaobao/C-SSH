//! 同步用例边界；账号与设备身份始终来自已认证会话。

use std::sync::Arc;

use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    AdminSyncRecord, ChangeDataProtectionRequest, DataProtectionMutationResponse,
    DataProtectionView, HostView, LegacyPullRequest, LegacyPullResponse,
    MigrateDataProtectionRequest, ProtectionResetChallengeRequest,
    ProtectionResetChallengeResponse, ProtectionResetMailer, PullAckRequest, PullRequest,
    PullResponse, PushOutcome, PushRequest, RekeySyncRequest, RekeySyncResponse, ResetSyncRequest,
    ResetSyncResponse, SetupDataProtectionRequest, SyncStateView,
    VerifyProtectionResetChallengeRequest, VerifyProtectionResetChallengeResponse,
    actor::{AccountActor, DeviceActor},
    protection_mailer::UnavailableProtectionResetMailer,
    repository, validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
    protection_verification: Arc<ProtectionVerification>,
}

struct ProtectionVerification {
    key: Vec<u8>,
    mailer: Arc<dyn ProtectionResetMailer>,
}

impl Drop for ProtectionVerification {
    fn drop(&mut self) {
        self.key.fill(0);
    }
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            protection_verification: Arc::new(ProtectionVerification {
                key: Vec::new(),
                mailer: Arc::new(UnavailableProtectionResetMailer),
            }),
        }
    }

    pub fn with_protection_verification(
        pool: PgPool,
        key: Vec<u8>,
        mailer: Arc<dyn ProtectionResetMailer>,
    ) -> AppResult<Self> {
        if key.len() < 32 {
            return Err(AppError::Validation(
                "protection reset verification key must contain at least 32 bytes".to_owned(),
            ));
        }
        Ok(Self {
            pool,
            protection_verification: Arc::new(ProtectionVerification { key, mailer }),
        })
    }

    pub async fn list_self(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<HostView>> {
        let actor = AccountActor::from_session(session)?;
        repository::list(&self.pool, actor.account_id(), page).await
    }

    pub async fn get_self(
        &self,
        session: &AuthenticatedSession,
        host_id: Uuid,
    ) -> AppResult<HostView> {
        let actor = AccountActor::from_session(session)?;
        validation::host_id(host_id)?;
        repository::get(&self.pool, actor.account_id(), host_id).await
    }

    pub async fn push(
        &self,
        session: &AuthenticatedSession,
        request: PushRequest,
    ) -> AppResult<PushOutcome> {
        let actor = DeviceActor::from_session(session)?;
        let changes = validation::push(&request)?;
        let request_hash = fingerprint("encrypted-sync-push-v2", &request)?;
        repository::push(&self.pool, actor, &request, &changes, &request_hash).await
    }

    pub async fn pull(
        &self,
        session: &AuthenticatedSession,
        request: PullRequest,
    ) -> AppResult<PullResponse> {
        let actor = DeviceActor::from_session(session)?;
        repository::pull(&self.pool, actor, validation::pull(request)?).await
    }

    pub async fn acknowledge_pull(
        &self,
        session: &AuthenticatedSession,
        request: PullAckRequest,
    ) -> AppResult<()> {
        let actor = DeviceActor::from_session(session)?;
        validation::ack(&request)?;
        repository::ack(&self.pool, actor, &request).await
    }

    pub async fn sync_state(&self, session: &AuthenticatedSession) -> AppResult<SyncStateView> {
        let actor = DeviceActor::from_session(session)?;
        repository::get_protection(&self.pool, actor)
            .await
            .map(|view| view.state)
    }

    pub async fn data_protection(
        &self,
        session: &AuthenticatedSession,
    ) -> AppResult<DataProtectionView> {
        let actor = DeviceActor::from_session(session)?;
        repository::get_protection(&self.pool, actor).await
    }

    pub async fn setup_data_protection(
        &self,
        session: &AuthenticatedSession,
        request: SetupDataProtectionRequest,
    ) -> AppResult<DataProtectionMutationResponse> {
        let actor = DeviceActor::from_session(session)?;
        let envelope = validation::setup_protection(&request)?;
        let request_hash = fingerprint("data-protection-setup-v1", &request)?;
        repository::setup_protection(&self.pool, actor, &request, &envelope, &request_hash).await
    }

    pub async fn migrate_data_protection(
        &self,
        session: &AuthenticatedSession,
        request: MigrateDataProtectionRequest,
    ) -> AppResult<DataProtectionMutationResponse> {
        let actor = DeviceActor::from_session(session)?;
        let (envelope, resources) = validation::migrate_protection(&request)?;
        let request_hash = fingerprint("data-protection-migrate-v1", &request)?;
        repository::migrate_protection(
            &self.pool,
            actor,
            &request,
            &envelope,
            &resources,
            &request_hash,
        )
        .await
    }

    pub async fn change_data_protection(
        &self,
        session: &AuthenticatedSession,
        request: ChangeDataProtectionRequest,
    ) -> AppResult<DataProtectionMutationResponse> {
        let actor = DeviceActor::from_session(session)?;
        let envelope = validation::change_protection(&request)?;
        let request_hash = fingerprint("data-protection-change-v1", &request)?;
        repository::change_protection(&self.pool, actor, &request, &envelope, &request_hash).await
    }

    pub async fn legacy_protection_pull(
        &self,
        session: &AuthenticatedSession,
        request: LegacyPullRequest,
    ) -> AppResult<LegacyPullResponse> {
        let actor = DeviceActor::from_session(session)?;
        repository::legacy_pull(&self.pool, actor, validation::legacy_pull(request)?).await
    }

    pub async fn request_protection_reset_challenge(
        &self,
        session: &AuthenticatedSession,
        _request: ProtectionResetChallengeRequest,
    ) -> AppResult<ProtectionResetChallengeResponse> {
        let actor = DeviceActor::from_session(session)?;
        let pending =
            repository::issue_reset_challenge(&self.pool, actor, &self.protection_verification.key)
                .await?;
        if let Err(error) = self
            .protection_verification
            .mailer
            .send_protection_reset(&pending.email, pending.code.expose())
            .await
        {
            let _ = repository::cancel_challenge(&self.pool, actor, pending.response.challenge_id)
                .await;
            return Err(error);
        }
        repository::mark_challenge_sent(&self.pool, actor, pending.response.challenge_id).await?;
        Ok(pending.response)
    }

    pub async fn verify_protection_reset_challenge(
        &self,
        session: &AuthenticatedSession,
        request: VerifyProtectionResetChallengeRequest,
    ) -> AppResult<VerifyProtectionResetChallengeResponse> {
        let actor = DeviceActor::from_session(session)?;
        validation::verify_reset_challenge(&request)?;
        repository::verify_reset_challenge(
            &self.pool,
            actor,
            &request,
            &self.protection_verification.key,
        )
        .await
    }

    pub async fn reset_sync(
        &self,
        session: &AuthenticatedSession,
        request: ResetSyncRequest,
    ) -> AppResult<ResetSyncResponse> {
        let actor = DeviceActor::from_session(session)?;
        validation::reset(&request)?;
        let request_hash = fingerprint("data-protection-reset-v1", &request)?;
        repository::reset(
            &self.pool,
            actor,
            &request,
            &request_hash,
            &self.protection_verification.key,
        )
        .await
    }

    pub async fn rekey_sync(
        &self,
        session: &AuthenticatedSession,
        request: RekeySyncRequest,
    ) -> AppResult<RekeySyncResponse> {
        let actor = DeviceActor::from_session(session)?;
        let hosts = validation::rekey(&request)?;
        let request_hash = fingerprint("encrypted-sync-rekey-v2", &request)?;
        repository::rekey(&self.pool, actor, &request, &hosts, &request_hash).await
    }

    pub async fn admin_count_for_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
    ) -> AppResult<i64> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        repository::count(&self.pool, account_id).await
    }

    pub async fn admin_list_for_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        page: PageQuery,
    ) -> AppResult<Page<HostView>> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        repository::list(&self.pool, account_id, page).await
    }

    pub async fn admin_get_for_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        host_id: Uuid,
    ) -> AppResult<HostView> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        validation::host_id(host_id)?;
        repository::get(&self.pool, account_id, host_id).await
    }

    pub async fn admin_delete_for_user(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        host_id: Uuid,
    ) -> AppResult<()> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        validation::host_id(host_id)?;
        repository::delete_admin_host(&self.pool, actor, account_id, host_id).await
    }

    pub async fn admin_list_sync_records(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        page: PageQuery,
    ) -> AppResult<Page<AdminSyncRecord>> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        repository::list_admin_sync_records(&self.pool, account_id, page).await
    }

    pub async fn admin_delete_sync_record(
        &self,
        actor: &AdminActor,
        account_id: Uuid,
        record_id: &str,
    ) -> AppResult<()> {
        require_admin(actor)?;
        validate_account_id(account_id)?;
        repository::delete_admin_sync_record(&self.pool, actor, account_id, record_id).await
    }
}

fn fingerprint<T: Serialize>(scope: &str, value: &T) -> AppResult<[u8; 32]> {
    let mut encoded = serde_json::to_vec(value)
        .map_err(|_| AppError::Internal("host sync request fingerprint failed".to_owned()))?;
    let mut hasher = Sha256::new();
    hasher.update(scope.as_bytes());
    hasher.update([0]);
    hasher.update(&encoded);
    let result = hasher.finalize().into();
    encoded.fill(0);
    Ok(result)
}

fn require_admin(actor: &AdminActor) -> AppResult<()> {
    if actor.account_id().is_nil() {
        Err(AppError::Unauthorized(
            "administrator identity is invalid".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn validate_account_id(account_id: Uuid) -> AppResult<()> {
    if account_id.is_nil() {
        Err(AppError::Validation(
            "account_id cannot be empty".to_owned(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{HostChange, HostMetadataInput, HostOperation, HostStatus, PushRequest};

    #[test]
    fn fingerprint_is_scoped_and_stable() {
        let value = json!({"revision": 7, "operation": "update"});
        let first = fingerprint("push", &value).expect("fingerprint");
        let repeated = fingerprint("push", &value).expect("fingerprint");
        let other_scope = fingerprint("resolve", &value).expect("fingerprint");
        assert_eq!(first, repeated);
        assert_ne!(first, other_scope);
    }

    #[test]
    fn fingerprint_distinguishes_missing_and_explicit_null_ciphertext() {
        let host_id = Uuid::now_v7();
        let metadata = HostMetadataInput {
            address: "node.example.com".to_owned(),
            port: 22,
            name: "node".to_owned(),
            platform: "linux".to_owned(),
            tags: vec![],
            status: HostStatus::Active,
        };
        let request = |ciphertext| PushRequest {
            sync_generation: 1,
            protection_epoch: 1,
            protection_revision: 1,
            base_revision: 1,
            client_mutation_id: Uuid::nil(),
            host_changes: vec![HostChange {
                host_id,
                operation: HostOperation::Update,
                metadata: Some(metadata.clone()),
                ciphertext,
                expected_revision: Some(1),
            }],
            ai_changes: vec![],
        };
        let missing =
            fingerprint("encrypted-sync-push-v2", &request(None)).expect("missing fingerprint");
        let clear = fingerprint("encrypted-sync-push-v2", &request(Some(None)))
            .expect("explicit null fingerprint");
        let replace = fingerprint(
            "encrypted-sync-push-v2",
            &request(Some(Some("AA==".to_owned()))),
        )
        .expect("replacement fingerprint");
        assert_ne!(missing, clear);
        assert_ne!(clear, replace);
        assert_ne!(missing, replace);
    }
}
