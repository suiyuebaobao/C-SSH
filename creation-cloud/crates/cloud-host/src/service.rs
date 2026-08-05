//! Use-case boundary. Identity is always derived from the authenticated session.

use cloud_domain::{AdminActor, AppError, AppResult, AuthenticatedSession, Page, PageQuery};
use cloud_store::PgPool;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    AdminSyncRecord, HostConflictView, HostView, PullAckRequest, PullRequest, PullResponse,
    PushOutcome, PushRequest, RekeySyncRequest, RekeySyncResponse, ResetSyncRequest,
    ResetSyncResponse, ResolveConflictOutcome, ResolveConflictRequest, SyncStateView,
    actor::{AccountActor, DeviceActor},
    repository, validation,
};

#[derive(Clone)]
pub struct Service {
    pool: PgPool,
}

impl Service {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
        let changes = validation::push(
            request.sync_generation,
            request.base_revision,
            request.client_mutation_id,
            &request.changes,
        )?;
        let request_hash = fingerprint("host-sync-push-v1", &request)?;
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

    pub async fn list_open_conflicts(
        &self,
        session: &AuthenticatedSession,
        page: PageQuery,
    ) -> AppResult<Page<HostConflictView>> {
        let actor = AccountActor::from_session(session)?;
        repository::list_open_conflicts(&self.pool, actor.account_id(), page).await
    }

    pub async fn get_conflict(
        &self,
        session: &AuthenticatedSession,
        conflict_id: Uuid,
    ) -> AppResult<HostConflictView> {
        let actor = AccountActor::from_session(session)?;
        validation::conflict_id(conflict_id)?;
        repository::get_conflict(&self.pool, actor.account_id(), conflict_id).await
    }

    pub async fn resolve_conflict(
        &self,
        session: &AuthenticatedSession,
        conflict_id: Uuid,
        request: ResolveConflictRequest,
    ) -> AppResult<ResolveConflictOutcome> {
        let actor = DeviceActor::from_session(session)?;
        validation::resolve(conflict_id, &request)?;
        let request_hash =
            fingerprint("host-sync-conflict-resolution-v1", &(conflict_id, &request))?;
        repository::resolve_conflict(&self.pool, actor, conflict_id, &request, &request_hash).await
    }

    pub async fn sync_state(&self, session: &AuthenticatedSession) -> AppResult<SyncStateView> {
        let actor = DeviceActor::from_session(session)?;
        repository::state(&self.pool, actor).await
    }

    pub async fn reset_sync(
        &self,
        session: &AuthenticatedSession,
        request: ResetSyncRequest,
    ) -> AppResult<ResetSyncResponse> {
        let actor = DeviceActor::from_session(session)?;
        validation::reset(&request)?;
        repository::reset(&self.pool, actor, &request).await
    }

    pub async fn rekey_sync(
        &self,
        session: &AuthenticatedSession,
        request: RekeySyncRequest,
    ) -> AppResult<RekeySyncResponse> {
        let actor = DeviceActor::from_session(session)?;
        let hosts = validation::rekey(&request)?;
        let request_hash = fingerprint("host-sync-rekey-v1", &request)?;
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
    let encoded = serde_json::to_vec(value)
        .map_err(|_| AppError::Internal("host sync request fingerprint failed".to_owned()))?;
    let mut hasher = Sha256::new();
    hasher.update(scope.as_bytes());
    hasher.update([0]);
    hasher.update(encoded);
    Ok(hasher.finalize().into())
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
            base_revision: 1,
            client_mutation_id: Uuid::nil(),
            changes: vec![HostChange {
                host_id,
                operation: HostOperation::Update,
                metadata: Some(metadata.clone()),
                ciphertext,
                expected_revision: Some(1),
            }],
        };
        let missing =
            fingerprint("host-sync-push-v1", &request(None)).expect("missing fingerprint");
        let clear = fingerprint("host-sync-push-v1", &request(Some(None)))
            .expect("explicit null fingerprint");
        let replace = fingerprint("host-sync-push-v1", &request(Some(Some("AA==".to_owned()))))
            .expect("replacement fingerprint");
        assert_ne!(missing, clear);
        assert_ne!(clear, replace);
        assert_ne!(missing, replace);
    }
}
