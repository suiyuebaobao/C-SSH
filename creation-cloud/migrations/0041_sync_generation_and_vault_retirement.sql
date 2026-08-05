-- A reset advances an account-scoped generation so an older device cannot
-- repopulate ciphertext that the user explicitly made unrecoverable.
ALTER TABLE cloud_host_sync_states
    ADD COLUMN sync_generation BIGINT NOT NULL DEFAULT 1,
    ADD CONSTRAINT cloud_host_sync_states_generation_positive
        CHECK (sync_generation > 0);

CREATE TABLE cloud_sync_reset_mutations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    result_generation BIGINT NOT NULL CHECK (result_generation > 1),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mutation_id),
    UNIQUE (account_id, result_generation),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (mutation_id <> '00000000-0000-0000-0000-000000000000'::uuid)
);

CREATE INDEX cloud_sync_reset_mutations_account_created_idx
    ON cloud_sync_reset_mutations(account_id, created_at DESC, mutation_id);

CREATE TABLE cloud_sync_rekey_mutations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    request_generation BIGINT NOT NULL CHECK (request_generation > 0),
    result_generation BIGINT NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    result_revision BIGINT NOT NULL CHECK (result_revision >= 0),
    changed_count INTEGER NOT NULL CHECK (changed_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mutation_id),
    UNIQUE (account_id, result_generation),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (mutation_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    CHECK (result_generation = request_generation + 1)
);

CREATE TABLE cloud_sync_rekey_results (
    account_id UUID NOT NULL,
    mutation_id UUID NOT NULL,
    host_id UUID NOT NULL,
    previous_revision BIGINT NOT NULL CHECK (previous_revision > 0),
    result_revision BIGINT NOT NULL CHECK (result_revision > previous_revision),
    PRIMARY KEY (account_id, mutation_id, host_id),
    FOREIGN KEY (account_id, mutation_id)
        REFERENCES cloud_sync_rekey_mutations(account_id, mutation_id)
        ON DELETE CASCADE
);

CREATE INDEX cloud_sync_rekey_mutations_account_created_idx
    ON cloud_sync_rekey_mutations(account_id, created_at DESC, mutation_id);

-- The reset audit is intentionally restricted to identity, counts and
-- generations. Host metadata, ciphertext and password-derived values cannot
-- be added by a future caller without an explicit forward migration.
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_sync_reset_semantic_contract CHECK (
    action <> 'sync.encrypted_data_reset'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'mutation_id', 'device_id',
            'removed_hosts', 'removed_versions', 'removed_conflicts',
            'removed_deliveries', 'removed_ack_records', 'removed_tombstones',
            'previous_sync_generation', 'sync_generation'
        ]
        AND details - ARRAY[
            'mutation_id', 'device_id',
            'removed_hosts', 'removed_versions', 'removed_conflicts',
            'removed_deliveries', 'removed_ack_records', 'removed_tombstones',
            'previous_sync_generation', 'sync_generation'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'removed_hosts') = 'number'
        AND jsonb_typeof(details->'removed_versions') = 'number'
        AND jsonb_typeof(details->'removed_conflicts') = 'number'
        AND jsonb_typeof(details->'removed_deliveries') = 'number'
        AND jsonb_typeof(details->'removed_ack_records') = 'number'
        AND jsonb_typeof(details->'removed_tombstones') = 'number'
        AND jsonb_typeof(details->'previous_sync_generation') = 'number'
        AND jsonb_typeof(details->'sync_generation') = 'number'
        AND (details->>'previous_sync_generation')::BIGINT > 0
        AND (details->>'sync_generation')::BIGINT
            = (details->>'previous_sync_generation')::BIGINT + 1
    )
);

ALTER TABLE audit_events
ADD CONSTRAINT audit_events_sync_rekey_semantic_contract CHECK (
    action <> 'sync.encrypted_data_rekey'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'mutation_id', 'device_id', 'changed_hosts', 'result_revision',
            'previous_sync_generation', 'sync_generation'
        ]
        AND details - ARRAY[
            'mutation_id', 'device_id', 'changed_hosts', 'result_revision',
            'previous_sync_generation', 'sync_generation'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'changed_hosts') = 'number'
        AND jsonb_typeof(details->'result_revision') = 'number'
        AND jsonb_typeof(details->'previous_sync_generation') = 'number'
        AND jsonb_typeof(details->'sync_generation') = 'number'
        AND (details->>'previous_sync_generation')::BIGINT > 0
        AND (details->>'sync_generation')::BIGINT
            = (details->>'previous_sync_generation')::BIGINT + 1
    )
);

-- Key/wrapper/envelope are no longer product or API concepts. The host
-- ciphertext tables remain and continue to store only client-produced bytes.
DROP TABLE IF EXISTS vault_key_wrappers;
DROP TABLE IF EXISTS vault_envelopes;
