-- 迁移不得解析、改写或删除既有地址；活动 hostname 以固定脱敏错误阻断整次迁移。
DO $creation_sync_ip_precheck$
DECLARE
    candidate RECORD;
    parsed INET;
BEGIN
    FOR candidate IN
        SELECT address FROM cloud_hosts WHERE NOT is_deleted
    LOOP
        BEGIN
            IF candidate.address <> btrim(candidate.address)
                OR position('/' IN candidate.address) > 0
                OR (
                    position(':' IN candidate.address) = 0
                    AND candidate.address !~ '^[0-9]{1,3}([.][0-9]{1,3}){3}$'
                )
                OR (
                    position(':' IN candidate.address) > 0
                    AND candidate.address !~ '^[0-9A-Fa-f:.]+$'
                )
            THEN
                RAISE EXCEPTION 'invalid active host address';
            END IF;
            parsed := candidate.address::inet;
            IF family(parsed) NOT IN (4, 6)
                OR (family(parsed) = 4 AND masklen(parsed) <> 32)
                OR (family(parsed) = 6 AND masklen(parsed) <> 128)
            THEN
                RAISE EXCEPTION 'invalid active host address';
            END IF;
        EXCEPTION WHEN OTHERS THEN
            RAISE EXCEPTION USING
                ERRCODE = '23514',
                MESSAGE = 'unified encrypted sync migration blocked: active host address must be numeric IP';
        END;
    END LOOP;
END;
$creation_sync_ip_precheck$;

-- 将主机和 AI provider 配置纳入同一账号 revision 流，并删除专用主机冲突持久层。

CREATE TABLE cloud_ai_provider_configs (
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
    CHECK (
        envelope_metadata IS NULL
        OR (
            jsonb_typeof(envelope_metadata) = 'object'
            AND pg_column_size(envelope_metadata) <= 16384
        )
    ),
    CHECK (
        (is_deleted AND ciphertext IS NULL AND nonce IS NULL AND envelope_metadata IS NULL)
        OR
        (NOT is_deleted AND ciphertext IS NOT NULL AND nonce IS NOT NULL
            AND envelope_metadata IS NOT NULL)
    )
);

CREATE INDEX cloud_ai_provider_configs_account_list_idx
    ON cloud_ai_provider_configs(account_id, is_deleted, updated_at DESC, id);
CREATE INDEX cloud_ai_provider_configs_revision_idx
    ON cloud_ai_provider_configs(account_id, revision, id);

CREATE TABLE cloud_ai_provider_config_versions (
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
        REFERENCES cloud_ai_provider_configs(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (ciphertext IS NULL OR octet_length(ciphertext) <= 262144),
    CHECK (nonce IS NULL OR octet_length(nonce) BETWEEN 1 AND 4096),
    CHECK (
        envelope_metadata IS NULL
        OR (
            jsonb_typeof(envelope_metadata) = 'object'
            AND pg_column_size(envelope_metadata) <= 16384
        )
    ),
    CHECK (
        (is_deleted AND ciphertext IS NULL AND nonce IS NULL AND envelope_metadata IS NULL)
        OR
        (NOT is_deleted AND ciphertext IS NOT NULL AND nonce IS NOT NULL
            AND envelope_metadata IS NOT NULL)
    )
);

CREATE INDEX cloud_ai_provider_versions_pull_idx
    ON cloud_ai_provider_config_versions(account_id, resource_id, revision DESC);
CREATE INDEX cloud_ai_provider_versions_retention_idx
    ON cloud_ai_provider_config_versions(recorded_at, account_id, revision);
CREATE INDEX cloud_ai_provider_tombstone_retention_idx
    ON cloud_ai_provider_configs(updated_at, account_id, revision, id)
    WHERE is_deleted;

CREATE INDEX cloud_host_versions_retention_idx
    ON cloud_host_versions(recorded_at, account_id, revision);
CREATE INDEX cloud_hosts_tombstone_retention_idx
    ON cloud_hosts(updated_at, account_id, revision, id)
    WHERE is_deleted;

CREATE TABLE cloud_sync_push_mutations_v2 (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    client_mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    request_generation BIGINT NOT NULL CHECK (request_generation > 0),
    base_revision BIGINT NOT NULL CHECK (base_revision >= 0),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    outcome TEXT NOT NULL CHECK (outcome IN ('applied', 'unchanged')),
    result_revision BIGINT NOT NULL CHECK (result_revision >= 0),
    changed_count INTEGER NOT NULL DEFAULT 0 CHECK (changed_count >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    admin_deleted_at TIMESTAMPTZ,
    PRIMARY KEY (account_id, client_mutation_id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (
        (outcome = 'applied' AND changed_count > 0)
        OR (outcome = 'unchanged' AND changed_count = 0)
    )
);

-- 旧请求正文与 v2 DTO 不同，只保留成功历史供后台展示，不提供兼容回放。
INSERT INTO cloud_sync_push_mutations_v2
    (account_id, client_mutation_id, source_device_id, request_generation,
     base_revision, request_hash, outcome, result_revision, changed_count,
     created_at, admin_deleted_at)
SELECT mutations.account_id, mutations.client_mutation_id,
       mutations.source_device_id, states.sync_generation, 0,
       mutations.request_hash, mutations.outcome, mutations.result_revision,
       mutations.changed_count, mutations.created_at, mutations.admin_deleted_at
FROM cloud_host_mutations AS mutations
JOIN cloud_host_sync_states AS states ON states.account_id = mutations.account_id
WHERE mutations.outcome IN ('applied', 'unchanged');

DROP TABLE cloud_host_mutations;
DROP TABLE cloud_host_conflicts;
ALTER TABLE cloud_sync_push_mutations_v2 RENAME TO cloud_sync_push_mutations;

CREATE INDEX cloud_sync_push_mutations_admin_visible_idx
    ON cloud_sync_push_mutations(account_id, created_at DESC, client_mutation_id DESC)
    WHERE admin_deleted_at IS NULL;
CREATE INDEX cloud_sync_push_mutations_retention_idx
    ON cloud_sync_push_mutations(account_id, created_at, client_mutation_id);

CREATE TABLE cloud_sync_push_results (
    account_id UUID NOT NULL,
    client_mutation_id UUID NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('host', 'ai_provider_account')),
    resource_id UUID NOT NULL,
    result_revision BIGINT NOT NULL CHECK (result_revision > 0),
    PRIMARY KEY (account_id, client_mutation_id, resource_kind, resource_id),
    FOREIGN KEY (account_id, client_mutation_id)
        REFERENCES cloud_sync_push_mutations(account_id, client_mutation_id)
        ON DELETE CASCADE
);

CREATE TABLE cloud_sync_resource_deliveries (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('host', 'ai_provider_account')),
    resource_id UUID NOT NULL,
    delivered_revision BIGINT NOT NULL CHECK (delivered_revision > 0),
    snapshot_revision BIGINT NOT NULL CHECK (snapshot_revision >= delivered_revision),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (
        account_id, device_id, resource_kind, resource_id,
        delivered_revision, snapshot_revision
    ),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE
);

INSERT INTO cloud_sync_resource_deliveries
    (account_id, device_id, resource_kind, resource_id,
     delivered_revision, snapshot_revision, updated_at)
SELECT account_id, device_id, 'host', host_id,
       delivered_revision, delivered_revision, updated_at
FROM cloud_host_device_deliveries
WHERE delivered_revision > 0;
DROP TABLE cloud_host_device_deliveries;

CREATE TABLE cloud_sync_pull_decisions (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('host', 'ai_provider_account')),
    resource_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    action TEXT NOT NULL CHECK (action IN ('replace_local', 'keep_local')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id, resource_kind, resource_id, revision),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE
);

INSERT INTO cloud_sync_pull_decisions
    (account_id, device_id, resource_kind, resource_id, revision, action, recorded_at)
SELECT account_id, device_id, 'host', host_id, cloud_revision, action, recorded_at
FROM cloud_host_pull_decisions;
DROP TABLE cloud_host_pull_decisions;

ALTER TABLE cloud_host_pull_watermarks RENAME TO cloud_sync_pull_watermarks;
ALTER TABLE cloud_host_device_checkpoints RENAME TO cloud_sync_device_checkpoints;
ALTER INDEX cloud_host_device_checkpoints_admin_visible_idx
    RENAME TO cloud_sync_device_checkpoints_admin_visible_idx;

-- 旧 checkpoint 可能已越过“已预览但未显式选择”的主机；迁移时回退到首个未决 revision 之前。
WITH latest_resources AS (
    SELECT checkpoint.account_id, checkpoint.device_id,
           latest.host_id AS resource_id, latest.revision
    FROM cloud_sync_device_checkpoints AS checkpoint
    JOIN LATERAL (
        SELECT DISTINCT ON (versions.host_id)
               versions.host_id, versions.revision
        FROM cloud_host_versions AS versions
        WHERE versions.account_id = checkpoint.account_id
          AND versions.revision <= checkpoint.acknowledged_revision
        ORDER BY versions.host_id, versions.revision DESC
    ) AS latest ON TRUE
), first_unresolved AS (
    SELECT resource.account_id, resource.device_id, MIN(resource.revision) AS revision
    FROM latest_resources AS resource
    WHERE NOT EXISTS (
        SELECT 1 FROM cloud_sync_pull_decisions AS decision
        WHERE decision.account_id = resource.account_id
          AND decision.device_id = resource.device_id
          AND decision.resource_kind = 'host'
          AND decision.resource_id = resource.resource_id
          AND decision.revision = resource.revision
    )
    GROUP BY resource.account_id, resource.device_id
)
UPDATE cloud_sync_device_checkpoints AS checkpoint
SET acknowledged_revision = LEAST(
        checkpoint.acknowledged_revision,
        GREATEST(first_unresolved.revision - 1, 0)
    ),
    updated_at = now()
FROM first_unresolved
WHERE checkpoint.account_id = first_unresolved.account_id
  AND checkpoint.device_id = first_unresolved.device_id;

CREATE TABLE cloud_sync_rekey_resource_results (
    account_id UUID NOT NULL,
    mutation_id UUID NOT NULL,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('host', 'ai_provider_account')),
    resource_id UUID NOT NULL,
    previous_revision BIGINT NOT NULL CHECK (previous_revision > 0),
    result_revision BIGINT NOT NULL CHECK (result_revision > previous_revision),
    PRIMARY KEY (account_id, mutation_id, resource_kind, resource_id),
    FOREIGN KEY (account_id, mutation_id)
        REFERENCES cloud_sync_rekey_mutations(account_id, mutation_id)
        ON DELETE CASCADE
);

INSERT INTO cloud_sync_rekey_resource_results
    (account_id, mutation_id, resource_kind, resource_id,
     previous_revision, result_revision)
SELECT account_id, mutation_id, 'host', host_id, previous_revision, result_revision
FROM cloud_sync_rekey_results;
DROP TABLE cloud_sync_rekey_results;

-- v2 使用新 action，旧 action、旧约束和历史审计正文保持逐字不变。
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_sync_reset_v2_semantic_contract CHECK (
    action <> 'sync.encrypted_data_reset_v2'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'mutation_id', 'device_id', 'removed_hosts', 'removed_versions',
            'removed_ai_providers', 'removed_ai_versions',
            'removed_deliveries', 'removed_ack_records', 'removed_tombstones',
            'previous_sync_generation', 'sync_generation'
        ]
        AND details - ARRAY[
            'mutation_id', 'device_id', 'removed_hosts', 'removed_versions',
            'removed_ai_providers', 'removed_ai_versions',
            'removed_deliveries', 'removed_ack_records', 'removed_tombstones',
            'previous_sync_generation', 'sync_generation'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'removed_hosts') = 'number'
        AND jsonb_typeof(details->'removed_versions') = 'number'
        AND jsonb_typeof(details->'removed_ai_providers') = 'number'
        AND jsonb_typeof(details->'removed_ai_versions') = 'number'
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
ADD CONSTRAINT audit_events_sync_rekey_v2_semantic_contract CHECK (
    action <> 'sync.encrypted_data_rekey_v2'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'sync_account'
        AND resource_id = actor_account_id::TEXT
        AND outcome = 'success'
        AND details ?& ARRAY[
            'mutation_id', 'device_id', 'changed_hosts',
            'changed_ai_providers', 'result_revision',
            'previous_sync_generation', 'sync_generation'
        ]
        AND details - ARRAY[
            'mutation_id', 'device_id', 'changed_hosts',
            'changed_ai_providers', 'result_revision',
            'previous_sync_generation', 'sync_generation'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'mutation_id') = 'string'
        AND jsonb_typeof(details->'device_id') = 'string'
        AND jsonb_typeof(details->'changed_hosts') = 'number'
        AND jsonb_typeof(details->'changed_ai_providers') = 'number'
        AND jsonb_typeof(details->'result_revision') = 'number'
        AND jsonb_typeof(details->'previous_sync_generation') = 'number'
        AND jsonb_typeof(details->'sync_generation') = 'number'
        AND (details->>'previous_sync_generation')::BIGINT > 0
        AND (details->>'sync_generation')::BIGINT
            = (details->>'previous_sync_generation')::BIGINT + 1
    )
);
