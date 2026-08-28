//! Account-level data-protection state, envelope transitions and recovery challenge.

mod challenge;
mod core;
mod legacy;
mod projection;

pub(crate) use challenge::{
    cancel_challenge, consume_email_authorization, issue_reset_challenge, mark_challenge_sent,
    verify_reset_challenge,
};
pub(crate) use core::{change as change_protection, setup as setup_protection};
pub(crate) use legacy::{migrate as migrate_protection, pull as legacy_pull};
pub(crate) use projection::get as get_protection;

pub(super) use core::{
    DataProtectionOperation, PriorProtectionMutation, audit_mutation, clear_delivery_state,
    insert_envelope, load_prior_mutation, persist_mutation, purge_prior_ciphertext_versions,
    response,
};
