//! Version-aware response serialization for the retained v2 shape and current v4 boundary.

use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::{LEGACY_SYNC_CONTRACT_VERSION, LegacyPullResponse, PullResponse};

impl Serialize for PullResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let legacy = self.sync_contract_version == LEGACY_SYNC_CONTRACT_VERSION;
        let mut state =
            serializer.serialize_struct("PullResponse", if legacy { 10 } else { 12 })?;
        if !legacy {
            state.serialize_field("sync_contract_version", &self.sync_contract_version)?;
        }
        state.serialize_field("sync_generation", &self.sync_generation)?;
        state.serialize_field("protection_epoch", &self.protection_epoch)?;
        state.serialize_field("protection_revision", &self.protection_revision)?;
        state.serialize_field("purpose", &self.purpose)?;
        state.serialize_field("mode", &self.mode)?;
        state.serialize_field("host_records", &self.host_records)?;
        state.serialize_field("ai_records", &self.ai_records)?;
        if !legacy {
            state.serialize_field("proxy_profile_records", &self.proxy_profile_records)?;
        }
        state.serialize_field("snapshot_revision", &self.snapshot_revision)?;
        state.serialize_field("next_revision", &self.next_revision)?;
        state.serialize_field("has_more", &self.has_more)?;
        state.end()
    }
}

impl Serialize for LegacyPullResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let legacy = self.sync_contract_version == LEGACY_SYNC_CONTRACT_VERSION;
        let mut state =
            serializer.serialize_struct("LegacyPullResponse", if legacy { 8 } else { 10 })?;
        if !legacy {
            state.serialize_field("sync_contract_version", &self.sync_contract_version)?;
        }
        state.serialize_field("sync_generation", &self.sync_generation)?;
        state.serialize_field("protection_epoch", &self.protection_epoch)?;
        state.serialize_field("protection_revision", &self.protection_revision)?;
        state.serialize_field("snapshot_revision", &self.snapshot_revision)?;
        state.serialize_field("host_records", &self.host_records)?;
        state.serialize_field("ai_records", &self.ai_records)?;
        if !legacy {
            state.serialize_field("proxy_profile_records", &self.proxy_profile_records)?;
        }
        state.serialize_field("next_cursor", &self.next_cursor)?;
        state.serialize_field("has_more", &self.has_more)?;
        state.end()
    }
}
