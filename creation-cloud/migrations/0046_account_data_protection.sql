-- Account data protection is an opaque client-side envelope. This migration
-- never derives a key, invents an envelope, or rewrites existing ciphertext.
-- Accounts with active 0.7.6 ciphertext and no envelope therefore remain at
-- protection 0/0 and are projected as legacy_migration_required by the API.

ALTER TABLE cloud_host_sync_states
    ADD COLUMN protection_epoch BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN protection_revision BIGINT NOT NULL DEFAULT 0,
    ADD CONSTRAINT cloud_host_sync_states_protection_nonnegative CHECK (
        protection_epoch >= 0 AND protection_revision >= 0
    ),
    ADD CONSTRAINT cloud_host_sync_states_protection_initial_pair CHECK (
        (protection_epoch = 0 AND protection_revision = 0)
        OR (protection_epoch > 0 AND protection_revision > 0)
    ),
    ADD CONSTRAINT cloud_host_sync_states_protection_identity UNIQUE (
        account_id, sync_generation, protection_epoch, protection_revision
    );

CREATE TABLE cloud_data_protection_envelopes (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    sync_generation BIGINT NOT NULL CHECK (sync_generation > 0),
    protection_epoch BIGINT NOT NULL CHECK (protection_epoch > 0),
    protection_revision BIGINT NOT NULL CHECK (protection_revision > 0),
    format_version SMALLINT NOT NULL CHECK (format_version = 1),
    kdf_algorithm TEXT NOT NULL CHECK (kdf_algorithm = 'argon2id'),
    kdf_version INTEGER NOT NULL CHECK (kdf_version = 19),
    kdf_memory_kib INTEGER NOT NULL CHECK (kdf_memory_kib = 19456),
    kdf_iterations INTEGER NOT NULL CHECK (kdf_iterations = 2),
    kdf_parallelism INTEGER NOT NULL CHECK (kdf_parallelism = 1),
    kdf_output_length INTEGER NOT NULL CHECK (kdf_output_length = 32),
    salt BYTEA NOT NULL CHECK (octet_length(salt) = 16),
    nonce BYTEA NOT NULL CHECK (octet_length(nonce) = 24),
    wrapped_data_key BYTEA NOT NULL CHECK (octet_length(wrapped_data_key) = 48),
    source_device_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (
        account_id, sync_generation, protection_epoch, protection_revision
    ) REFERENCES cloud_host_sync_states (
        account_id, sync_generation, protection_epoch, protection_revision
    ) DEFERRABLE INITIALLY DEFERRED,
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE SET NULL (source_device_id)
);

CREATE INDEX cloud_data_protection_envelopes_device_idx
    ON cloud_data_protection_envelopes(account_id, source_device_id);

-- One table provides a single idempotency namespace for setup, legacy
-- migration, wrapper-only password change, and destructive reset.
CREATE TABLE cloud_data_protection_mutations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mutation_id UUID NOT NULL,
    operation TEXT NOT NULL
        CHECK (operation IN ('setup', 'migrate', 'change', 'reset')),
    source_device_id UUID NOT NULL,
    request_generation BIGINT NOT NULL CHECK (request_generation > 0),
    request_epoch BIGINT NOT NULL CHECK (request_epoch >= 0),
    request_revision BIGINT NOT NULL CHECK (request_revision >= 0),
    request_current_revision BIGINT NOT NULL CHECK (request_current_revision >= 0),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    result_generation BIGINT NOT NULL CHECK (result_generation > 0),
    result_epoch BIGINT NOT NULL CHECK (result_epoch > 0),
    result_revision BIGINT NOT NULL CHECK (result_revision > 0),
    result_current_revision BIGINT NOT NULL CHECK (result_current_revision >= 0),
    changed_count INTEGER NOT NULL DEFAULT 0 CHECK (changed_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mutation_id),
    UNIQUE (account_id, result_revision),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (mutation_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    CHECK (
        (operation IN ('setup', 'migrate', 'reset')
            AND result_generation = request_generation + 1)
        OR (operation = 'change' AND result_generation = request_generation)
    ),
    CHECK (
        (operation = 'setup'
            AND result_epoch = request_epoch + 1
            AND result_revision = request_revision + 1)
        OR (operation = 'migrate'
            AND request_epoch = 0 AND request_revision = 0
            AND result_epoch = 1 AND result_revision = 1)
        OR (operation = 'change'
            AND result_epoch = request_epoch
            AND result_revision = request_revision + 1)
        OR (operation = 'reset'
            AND result_epoch = request_epoch + 1
            AND result_revision = request_revision + 1)
    ),
    CHECK (
        (operation IN ('setup', 'change')
            AND result_current_revision = request_current_revision
            AND changed_count = 0)
        OR (operation = 'migrate'
            AND result_current_revision = request_current_revision + changed_count)
        OR (operation = 'reset'
            AND result_current_revision = 0)
    )
);

CREATE INDEX cloud_data_protection_mutations_retention_idx
    ON cloud_data_protection_mutations(account_id, created_at, mutation_id);

CREATE TABLE cloud_data_protection_migration_results (
    account_id UUID NOT NULL,
    mutation_id UUID NOT NULL,
    resource_kind TEXT NOT NULL
        CHECK (resource_kind IN ('host', 'ai_provider_account')),
    resource_id UUID NOT NULL,
    previous_revision BIGINT NOT NULL CHECK (previous_revision > 0),
    result_revision BIGINT NOT NULL CHECK (result_revision > previous_revision),
    PRIMARY KEY (account_id, mutation_id, resource_kind, resource_id),
    FOREIGN KEY (account_id, mutation_id)
        REFERENCES cloud_data_protection_mutations(account_id, mutation_id)
        ON DELETE CASCADE
);

-- A protection-reset code is independent from registration, login and account
-- password reset. A successful verification stores only a random authorization
-- token digest; the reset transaction consumes that receipt exactly once.
CREATE TABLE cloud_data_protection_reset_challenges (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    email TEXT NOT NULL,
    credential_version BIGINT NOT NULL CHECK (credential_version > 0),
    sync_generation BIGINT NOT NULL CHECK (sync_generation > 0),
    protection_epoch BIGINT NOT NULL CHECK (protection_epoch >= 0),
    protection_revision BIGINT NOT NULL CHECK (protection_revision >= 0),
    current_revision BIGINT NOT NULL CHECK (current_revision >= 0),
    code_digest BYTEA NOT NULL CHECK (octet_length(code_digest) = 32),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count BETWEEN 0 AND 5),
    authorization_digest BYTEA
        CHECK (authorization_digest IS NULL OR octet_length(authorization_digest) = 32),
    expires_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    CHECK (expires_at > created_at),
    CHECK (sent_at IS NULL OR sent_at >= created_at),
    CHECK (
        (verified_at IS NULL AND authorization_digest IS NULL)
        OR (verified_at IS NOT NULL AND authorization_digest IS NOT NULL)
    ),
    CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);

CREATE UNIQUE INDEX cloud_data_protection_reset_one_open_idx
    ON cloud_data_protection_reset_challenges(account_id)
    WHERE consumed_at IS NULL;
CREATE INDEX cloud_data_protection_reset_cleanup_idx
    ON cloud_data_protection_reset_challenges(account_id, expires_at, id)
    WHERE consumed_at IS NULL;
CREATE INDEX cloud_data_protection_reset_consumed_retention_idx
    ON cloud_data_protection_reset_challenges(account_id, consumed_at, id)
    WHERE consumed_at IS NOT NULL;

-- New pushes bind the protection counters. Historical v2 rows remain valid
-- admin/retention evidence but cannot be replayed as a protected request.
ALTER TABLE cloud_sync_push_mutations
    ADD COLUMN request_protection_epoch BIGINT NOT NULL DEFAULT 0
        CHECK (request_protection_epoch >= 0),
    ADD COLUMN request_protection_revision BIGINT NOT NULL DEFAULT 0
        CHECK (request_protection_revision >= 0);

-- Protection audit contains only identities, monotonic counters and counts.
-- The exact-key gate prevents wrappers, salts, codes or authorization tokens
-- from being added by a caller without a forward migration.
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_data_protection_v1_semantic_contract CHECK (
    action <> 'sync.data_protection_mutation_v1'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'operation', 'mutation_id', 'device_id', 'authorization_mode',
            'previous_sync_generation', 'sync_generation',
            'previous_protection_epoch', 'protection_epoch',
            'previous_protection_revision', 'protection_revision',
            'previous_current_revision', 'current_revision',
            'changed_count', 'removed_resource_count'
        ]
        AND details - ARRAY[
            'operation', 'mutation_id', 'device_id', 'authorization_mode',
            'previous_sync_generation', 'sync_generation',
            'previous_protection_epoch', 'protection_epoch',
            'previous_protection_revision', 'protection_revision',
            'previous_current_revision', 'current_revision',
            'changed_count', 'removed_resource_count'
        ] = '{}'::jsonb
        AND details->>'operation' IN ('setup', 'migrate', 'change', 'reset')
        AND details->>'authorization_mode' IN ('not_applicable', 'client_local_check', 'email_recovery')
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'previous_sync_generation') = 'number'
        AND jsonb_typeof(details->'sync_generation') = 'number'
        AND jsonb_typeof(details->'previous_protection_epoch') = 'number'
        AND jsonb_typeof(details->'protection_epoch') = 'number'
        AND jsonb_typeof(details->'previous_protection_revision') = 'number'
        AND jsonb_typeof(details->'protection_revision') = 'number'
        AND jsonb_typeof(details->'previous_current_revision') = 'number'
        AND jsonb_typeof(details->'current_revision') = 'number'
        AND jsonb_typeof(details->'changed_count') = 'number'
        AND jsonb_typeof(details->'removed_resource_count') = 'number'
    )
);
