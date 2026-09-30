//! 定义已校验 rekey 资源联合，供迁移与重包事务共享。

use super::ValidatedAiPayload;
use crate::ResourceKind;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub(crate) enum ValidatedRekeyResource {
    Host {
        resource_id: Uuid,
        cloud_revision: i64,
        ciphertext: Vec<u8>,
    },
    AiProviderAccount {
        resource_id: Uuid,
        cloud_revision: i64,
        payload: ValidatedAiPayload,
    },
    ProxyProfile {
        resource_id: Uuid,
        cloud_revision: i64,
        payload: ValidatedAiPayload,
    },
}

impl ValidatedRekeyResource {
    pub(crate) const fn resource_kind(&self) -> ResourceKind {
        match self {
            Self::Host { .. } => ResourceKind::Host,
            Self::AiProviderAccount { .. } => ResourceKind::AiProviderAccount,
            Self::ProxyProfile { .. } => ResourceKind::ProxyProfile,
        }
    }

    pub(crate) const fn resource_id(&self) -> Uuid {
        match self {
            Self::Host { resource_id, .. }
            | Self::AiProviderAccount { resource_id, .. }
            | Self::ProxyProfile { resource_id, .. } => *resource_id,
        }
    }

    pub(crate) const fn cloud_revision(&self) -> i64 {
        match self {
            Self::Host { cloud_revision, .. }
            | Self::AiProviderAccount { cloud_revision, .. }
            | Self::ProxyProfile { cloud_revision, .. } => *cloud_revision,
        }
    }
}
