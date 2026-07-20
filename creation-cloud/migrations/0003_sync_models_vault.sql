CREATE TABLE sync_states (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    current_revision BIGINT NOT NULL DEFAULT 0 CHECK (current_revision >= 0)
);

CREATE TABLE sync_records (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    namespace TEXT NOT NULL,
    record_key TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    value JSONB,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, namespace, record_key),
    CHECK ((is_deleted AND value IS NULL) OR (NOT is_deleted AND value IS NOT NULL))
);

CREATE INDEX sync_records_revision_idx
    ON sync_records(account_id, revision, namespace, record_key);

CREATE TABLE sync_conflicts (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    client_mutation_id UUID NOT NULL,
    base_revision BIGINT NOT NULL CHECK (base_revision >= 0),
    current_revision BIGINT NOT NULL CHECK (current_revision >= 0),
    attempted_changes JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ,
    UNIQUE (account_id, id),
    UNIQUE (account_id, client_mutation_id),
    CHECK (jsonb_typeof(attempted_changes) = 'array')
);

CREATE INDEX sync_conflicts_open_idx
    ON sync_conflicts(account_id, created_at DESC)
    WHERE resolved_at IS NULL;

CREATE TABLE sync_mutations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    client_mutation_id UUID NOT NULL,
    base_revision BIGINT NOT NULL CHECK (base_revision >= 0),
    mutation_hash TEXT NOT NULL CHECK (length(mutation_hash) = 64),
    outcome TEXT NOT NULL CHECK (outcome IN ('applied', 'conflict')),
    committed_revision BIGINT NOT NULL CHECK (committed_revision >= 0),
    conflict_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, client_mutation_id),
    FOREIGN KEY (account_id, conflict_id)
        REFERENCES sync_conflicts(account_id, id) ON DELETE CASCADE,
    CHECK (
        (outcome = 'applied' AND conflict_id IS NULL)
        OR (outcome = 'conflict' AND conflict_id IS NOT NULL)
    )
);

CREATE TABLE model_profiles (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    provider TEXT NOT NULL CHECK (length(provider) BETWEEN 1 AND 64),
    base_url TEXT,
    model_name TEXT NOT NULL CHECK (length(model_name) BETWEEN 1 AND 128),
    context_length INTEGER NOT NULL CHECK (context_length BETWEEN 256 AND 2000000),
    capability_tags TEXT[] NOT NULL DEFAULT '{}',
    default_parameters JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    vault_envelope_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, name),
    CHECK (jsonb_typeof(default_parameters) = 'object'),
    CHECK (enabled OR NOT is_default)
);

CREATE INDEX model_profiles_account_id_idx ON model_profiles(account_id);
CREATE UNIQUE INDEX model_profiles_one_default_idx
    ON model_profiles(account_id)
    WHERE is_default;

CREATE TABLE vault_envelopes (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    envelope_key UUID NOT NULL,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    schema_version INTEGER NOT NULL CHECK (schema_version BETWEEN 1 AND 65535),
    key_version INTEGER NOT NULL CHECK (key_version BETWEEN 1 AND 65535),
    cipher_suite TEXT NOT NULL CHECK (cipher_suite = 'xchacha20-poly1305'),
    kdf_metadata JSONB NOT NULL,
    nonce BYTEA NOT NULL CHECK (octet_length(nonce) = 24),
    ciphertext BYTEA NOT NULL CHECK (octet_length(ciphertext) BETWEEN 16 AND 1048576),
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, envelope_key),
    CHECK (jsonb_typeof(kdf_metadata) = 'object')
);

CREATE INDEX vault_envelopes_active_idx
    ON vault_envelopes(account_id, updated_at DESC)
    WHERE deleted_at IS NULL;
