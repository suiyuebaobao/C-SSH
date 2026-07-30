CREATE TABLE cloud_host_sync_states (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    current_revision BIGINT NOT NULL DEFAULT 0 CHECK (current_revision >= 0),
    compacted_through_revision BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        compacted_through_revision >= 0
        AND compacted_through_revision <= current_revision
    )
);

CREATE TABLE cloud_hosts (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    id UUID NOT NULL,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    name TEXT NOT NULL,
    platform TEXT NOT NULL,
    tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL,
    ciphertext BYTEA,
    source_device_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (char_length(address) BETWEEN 1 AND 253),
    CHECK (port BETWEEN 1 AND 65535),
    CHECK (char_length(name) BETWEEN 1 AND 128),
    CHECK (char_length(platform) BETWEEN 1 AND 32),
    CHECK (jsonb_typeof(tags) = 'array'),
    CHECK (status IN ('active', 'disabled', 'archived')),
    CHECK (ciphertext IS NULL OR octet_length(ciphertext) <= 262144),
    CHECK (NOT is_deleted OR ciphertext IS NULL)
);

CREATE INDEX cloud_hosts_account_list_idx
    ON cloud_hosts(account_id, is_deleted, updated_at DESC, id);
CREATE INDEX cloud_hosts_revision_idx
    ON cloud_hosts(account_id, revision, id);

CREATE TABLE cloud_host_versions (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    host_id UUID NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    name TEXT NOT NULL,
    platform TEXT NOT NULL,
    tags JSONB NOT NULL,
    status TEXT NOT NULL,
    ciphertext BYTEA,
    source_device_id UUID NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, revision),
    UNIQUE (account_id, host_id, revision),
    FOREIGN KEY (account_id, host_id)
        REFERENCES cloud_hosts(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (char_length(address) BETWEEN 1 AND 253),
    CHECK (port BETWEEN 1 AND 65535),
    CHECK (char_length(name) BETWEEN 1 AND 128),
    CHECK (char_length(platform) BETWEEN 1 AND 32),
    CHECK (jsonb_typeof(tags) = 'array'),
    CHECK (status IN ('active', 'disabled', 'archived')),
    CHECK (ciphertext IS NULL OR octet_length(ciphertext) <= 262144),
    CHECK (NOT is_deleted OR ciphertext IS NULL)
);

CREATE INDEX cloud_host_versions_pull_idx
    ON cloud_host_versions(account_id, host_id, revision DESC);

CREATE TABLE cloud_host_conflicts (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    host_id UUID NOT NULL,
    client_mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    base_revision BIGINT NOT NULL CHECK (base_revision >= 0),
    remote_revision BIGINT NOT NULL CHECK (remote_revision >= 0),
    proposed_operation TEXT NOT NULL
        CHECK (proposed_operation IN ('insert', 'update', 'delete')),
    proposed_address TEXT,
    proposed_port INTEGER,
    proposed_name TEXT,
    proposed_platform TEXT,
    proposed_tags JSONB,
    proposed_status TEXT,
    proposed_ciphertext_is_set BOOLEAN NOT NULL DEFAULT FALSE,
    proposed_ciphertext BYTEA,
    proposed_expected_revision BIGINT,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    resolution_action TEXT
        CHECK (resolution_action IN ('replace_remote', 'keep_remote')),
    resolution_mutation_id UUID,
    resolution_hash BYTEA,
    resolved_device_id UUID,
    resolved_revision BIGINT,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, client_mutation_id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    FOREIGN KEY (account_id, resolved_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (
        (proposed_operation = 'delete'
            AND proposed_address IS NULL
            AND proposed_port IS NULL
            AND proposed_name IS NULL
            AND proposed_platform IS NULL
            AND proposed_tags IS NULL
            AND proposed_status IS NULL
            AND NOT proposed_ciphertext_is_set
            AND proposed_ciphertext IS NULL
            AND proposed_expected_revision IS NOT NULL
            AND proposed_expected_revision > 0)
        OR
        (proposed_operation = 'insert'
            AND proposed_address IS NOT NULL
            AND char_length(proposed_address) BETWEEN 1 AND 253
            AND proposed_port BETWEEN 1 AND 65535
            AND proposed_name IS NOT NULL
            AND char_length(proposed_name) BETWEEN 1 AND 128
            AND proposed_platform IS NOT NULL
            AND char_length(proposed_platform) BETWEEN 1 AND 32
            AND proposed_tags IS NOT NULL
            AND jsonb_typeof(proposed_tags) = 'array'
            AND proposed_status IN ('active', 'disabled', 'archived')
            AND proposed_expected_revision IS NULL)
        OR
        (proposed_operation = 'update'
            AND proposed_address IS NOT NULL
            AND char_length(proposed_address) BETWEEN 1 AND 253
            AND proposed_port BETWEEN 1 AND 65535
            AND proposed_name IS NOT NULL
            AND char_length(proposed_name) BETWEEN 1 AND 128
            AND proposed_platform IS NOT NULL
            AND char_length(proposed_platform) BETWEEN 1 AND 32
            AND proposed_tags IS NOT NULL
            AND jsonb_typeof(proposed_tags) = 'array'
            AND proposed_status IN ('active', 'disabled', 'archived')
            AND proposed_expected_revision IS NOT NULL
            AND proposed_expected_revision > 0
        )
    ),
    CHECK (
        proposed_ciphertext_is_set
        OR proposed_ciphertext IS NULL
    ),
    CHECK (
        proposed_ciphertext IS NULL
        OR octet_length(proposed_ciphertext) <= 262144
    ),
    CHECK (
        (resolved_at IS NULL
            AND resolution_action IS NULL
            AND resolution_mutation_id IS NULL
            AND resolution_hash IS NULL
            AND resolved_device_id IS NULL
            AND resolved_revision IS NULL)
        OR
        (resolved_at IS NOT NULL
            AND resolution_action IS NOT NULL
            AND resolution_mutation_id IS NOT NULL
            AND resolution_hash IS NOT NULL
            AND octet_length(resolution_hash) = 32
            AND resolved_device_id IS NOT NULL
            AND resolved_revision IS NOT NULL
            AND resolved_revision >= 0)
    )
);

CREATE UNIQUE INDEX cloud_host_conflicts_resolution_mutation_idx
    ON cloud_host_conflicts(account_id, resolution_mutation_id)
    WHERE resolution_mutation_id IS NOT NULL;
CREATE INDEX cloud_host_conflicts_open_idx
    ON cloud_host_conflicts(account_id, created_at, id)
    WHERE resolved_at IS NULL;

CREATE TABLE cloud_host_mutations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    client_mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    outcome TEXT NOT NULL CHECK (outcome IN ('applied', 'unchanged', 'conflict')),
    result_revision BIGINT NOT NULL CHECK (result_revision >= 0),
    changed_count INTEGER NOT NULL DEFAULT 0 CHECK (changed_count >= 0),
    conflict_id UUID REFERENCES cloud_host_conflicts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, client_mutation_id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (
        (outcome = 'conflict' AND conflict_id IS NOT NULL)
        OR (outcome <> 'conflict' AND conflict_id IS NULL)
    ),
    CHECK (
        (outcome = 'applied' AND changed_count > 0)
        OR (outcome IN ('unchanged', 'conflict') AND changed_count = 0)
    )
);

CREATE TABLE cloud_host_download_allowlist (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    host_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id, host_id),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, host_id)
        REFERENCES cloud_hosts(account_id, id) ON DELETE CASCADE
);

CREATE INDEX cloud_host_download_allowlist_host_idx
    ON cloud_host_download_allowlist(account_id, host_id, device_id);

CREATE TABLE cloud_host_device_deliveries (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    host_id UUID NOT NULL,
    delivered_revision BIGINT NOT NULL DEFAULT 0 CHECK (delivered_revision >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id, host_id, delivered_revision),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, host_id)
        REFERENCES cloud_hosts(account_id, id) ON DELETE CASCADE
);

CREATE TABLE cloud_host_pull_watermarks (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    acknowledgeable_revision BIGINT NOT NULL CHECK (acknowledgeable_revision >= 0),
    snapshot_revision BIGINT NOT NULL CHECK (snapshot_revision >= acknowledgeable_revision),
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id, acknowledgeable_revision),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE
);

CREATE TABLE cloud_host_device_checkpoints (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    acknowledged_revision BIGINT NOT NULL DEFAULT 0 CHECK (acknowledged_revision >= 0),
    last_manual_sync_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE
);

CREATE TABLE cloud_host_pull_decisions (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    host_id UUID NOT NULL,
    cloud_revision BIGINT NOT NULL CHECK (cloud_revision > 0),
    action TEXT NOT NULL CHECK (action IN ('replace_local', 'keep_local')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id, host_id, cloud_revision),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (account_id, host_id)
        REFERENCES cloud_hosts(account_id, id) ON DELETE CASCADE
);
