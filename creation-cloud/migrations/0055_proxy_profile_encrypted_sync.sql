-- 第三方代理线路只以客户端不透明密文参与统一账号 revision 流。
ALTER TABLE cloud_host_sync_states
    ADD COLUMN minimum_sync_contract_version INTEGER NOT NULL DEFAULT 2
        CHECK (minimum_sync_contract_version IN (2, 3));

CREATE TABLE cloud_proxy_profiles (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    id UUID NOT NULL,
    ciphertext BYTEA,
    nonce BYTEA,
    envelope_metadata JSONB,
    source_device_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (ciphertext IS NULL OR octet_length(ciphertext) <= 262144),
    CHECK (nonce IS NULL OR octet_length(nonce) BETWEEN 1 AND 4096),
    CHECK (envelope_metadata IS NULL OR (
        jsonb_typeof(envelope_metadata) = 'object'
        AND pg_column_size(envelope_metadata) <= 16384
    )),
    CHECK (
        (is_deleted AND ciphertext IS NULL AND nonce IS NULL AND envelope_metadata IS NULL)
        OR (NOT is_deleted AND ciphertext IS NOT NULL AND nonce IS NOT NULL
            AND envelope_metadata IS NOT NULL)
    )
);

CREATE INDEX cloud_proxy_profiles_revision_idx
    ON cloud_proxy_profiles(account_id, revision, id);
CREATE INDEX cloud_proxy_profiles_tombstone_retention_idx
    ON cloud_proxy_profiles(updated_at, account_id, revision, id) WHERE is_deleted;

CREATE TABLE cloud_proxy_profile_versions (
    account_id UUID NOT NULL,
    resource_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    ciphertext BYTEA,
    nonce BYTEA,
    envelope_metadata JSONB,
    source_device_id UUID NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, revision),
    UNIQUE (account_id, resource_id, revision),
    FOREIGN KEY (account_id, resource_id)
        REFERENCES cloud_proxy_profiles(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (ciphertext IS NULL OR octet_length(ciphertext) <= 262144),
    CHECK (nonce IS NULL OR octet_length(nonce) BETWEEN 1 AND 4096),
    CHECK (envelope_metadata IS NULL OR (
        jsonb_typeof(envelope_metadata) = 'object'
        AND pg_column_size(envelope_metadata) <= 16384
    )),
    CHECK (
        (is_deleted AND ciphertext IS NULL AND nonce IS NULL AND envelope_metadata IS NULL)
        OR (NOT is_deleted AND ciphertext IS NOT NULL AND nonce IS NOT NULL
            AND envelope_metadata IS NOT NULL)
    )
);

CREATE INDEX cloud_proxy_profile_versions_pull_idx
    ON cloud_proxy_profile_versions(account_id, resource_id, revision DESC);
CREATE INDEX cloud_proxy_profile_versions_retention_idx
    ON cloud_proxy_profile_versions(recorded_at, account_id, revision);

ALTER TABLE cloud_sync_push_results
    DROP CONSTRAINT cloud_sync_push_results_resource_kind_check,
    ADD CONSTRAINT cloud_sync_push_results_resource_kind_check
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile'));
ALTER TABLE cloud_sync_resource_deliveries
    DROP CONSTRAINT cloud_sync_resource_deliveries_resource_kind_check,
    ADD CONSTRAINT cloud_sync_resource_deliveries_resource_kind_check
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile'));
ALTER TABLE cloud_sync_pull_decisions
    DROP CONSTRAINT cloud_sync_pull_decisions_resource_kind_check,
    ADD CONSTRAINT cloud_sync_pull_decisions_resource_kind_check
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile'));
ALTER TABLE cloud_sync_rekey_resource_results
    DROP CONSTRAINT cloud_sync_rekey_resource_results_resource_kind_check,
    ADD CONSTRAINT cloud_sync_rekey_resource_results_resource_kind_check
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile'));
ALTER TABLE cloud_data_protection_migration_results
    DROP CONSTRAINT cloud_data_protection_migration_results_resource_kind_check,
    ADD CONSTRAINT cloud_data_protection_migration_results_resource_kind_check
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile'));

ALTER TABLE audit_events
    DROP CONSTRAINT audit_events_sync_rekey_v2_semantic_contract;
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_sync_rekey_v2_semantic_contract CHECK (
    action <> 'sync.encrypted_data_rekey_v2'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'mutation_id','device_id','changed_hosts','changed_ai_providers',
            'result_revision','previous_sync_generation','sync_generation'
        ]
        AND details - ARRAY[
            'mutation_id','device_id','changed_hosts','changed_ai_providers',
            'changed_proxy_profiles','result_revision',
            'previous_sync_generation','sync_generation'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'changed_hosts') = 'number'
        AND jsonb_typeof(details->'changed_ai_providers') = 'number'
        -- 已发布的二资源审计正文不可回写；仅兼容缺少新增计数的完整旧形状。
        -- 新写入由同一 typed producer 始终提供 changed_proxy_profiles。
        AND (NOT (details ? 'changed_proxy_profiles')
            OR jsonb_typeof(details->'changed_proxy_profiles') = 'number')
        AND jsonb_typeof(details->'result_revision') = 'number'
        AND jsonb_typeof(details->'previous_sync_generation') = 'number'
        AND jsonb_typeof(details->'sync_generation') = 'number'
        AND (details->>'previous_sync_generation')::BIGINT > 0
        AND (details->>'sync_generation')::BIGINT
            = (details->>'previous_sync_generation')::BIGINT + 1
    )
);
