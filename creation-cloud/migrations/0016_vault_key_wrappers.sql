CREATE TABLE vault_key_wrappers (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    wrapper_key UUID NOT NULL,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    schema_version INTEGER NOT NULL CHECK (schema_version BETWEEN 1 AND 65535),
    key_version INTEGER NOT NULL CHECK (key_version BETWEEN 1 AND 65535),
    cipher_suite TEXT NOT NULL CHECK (cipher_suite = 'xchacha20-poly1305'),
    kdf_algorithm TEXT NOT NULL CHECK (kdf_algorithm = 'argon2id'),
    kdf_salt BYTEA NOT NULL CHECK (octet_length(kdf_salt) BETWEEN 16 AND 64),
    kdf_memory_kib BIGINT NOT NULL CHECK (kdf_memory_kib BETWEEN 19456 AND 1048576),
    kdf_iterations BIGINT NOT NULL CHECK (kdf_iterations BETWEEN 1 AND 20),
    kdf_parallelism BIGINT NOT NULL CHECK (kdf_parallelism BETWEEN 1 AND 16),
    nonce BYTEA NOT NULL CHECK (octet_length(nonce) = 24),
    wrapped_master_key_ciphertext BYTEA NOT NULL
        CHECK (octet_length(wrapped_master_key_ciphertext) = 48),
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT vault_key_wrappers_non_nil_key CHECK (
        wrapper_key <> '00000000-0000-0000-0000-000000000000'::uuid
    ),
    CONSTRAINT vault_key_wrappers_timestamp_order CHECK (
        updated_at >= created_at
        AND (deleted_at IS NULL OR deleted_at >= created_at)
    )
);

CREATE UNIQUE INDEX vault_key_wrappers_active_key_unique_idx
    ON vault_key_wrappers(account_id, wrapper_key)
    WHERE deleted_at IS NULL;

CREATE INDEX vault_key_wrappers_active_account_idx
    ON vault_key_wrappers(account_id, updated_at DESC, id DESC)
    WHERE deleted_at IS NULL;
