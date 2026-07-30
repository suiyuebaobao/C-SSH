-- 管理员维护的全局模型目录。旧 model_profiles 保留为只读 legacy，
-- 不自动公开任何账号级记录，也不再由活动路由读写。
CREATE TABLE global_model_catalog (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    provider TEXT NOT NULL CHECK (
        length(provider) BETWEEN 1 AND 64
        AND provider ~ '^[A-Za-z0-9._-]+$'
    ),
    base_url TEXT,
    model_name TEXT NOT NULL CHECK (length(model_name) BETWEEN 1 AND 128),
    context_length INTEGER NOT NULL CHECK (context_length BETWEEN 256 AND 2000000),
    capability_tags TEXT[] NOT NULL DEFAULT '{}',
    default_parameters JSONB NOT NULL DEFAULT '{}'
        CHECK (jsonb_typeof(default_parameters) = 'object'),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CHECK (enabled OR NOT is_default),
    CHECK (updated_at >= created_at),
    CHECK (deleted_at IS NULL OR deleted_at >= created_at)
);

CREATE UNIQUE INDEX global_model_catalog_active_name_unique_idx
    ON global_model_catalog (lower(name))
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX global_model_catalog_one_default_idx
    ON global_model_catalog ((is_default))
    WHERE deleted_at IS NULL AND enabled AND is_default;

CREATE INDEX global_model_catalog_public_order_idx
    ON global_model_catalog (is_default DESC, sort_order, name, id)
    WHERE deleted_at IS NULL AND enabled;

-- 个人 API key/Token 只以客户端产生的不透明密文保存。
-- 服务端不保存算法、nonce、KDF 或任何解密密钥。
CREATE TABLE account_model_secrets (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES global_model_catalog(id) ON DELETE RESTRICT,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    ciphertext BYTEA,
    source_device_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (account_id, model_id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE SET NULL (source_device_id),
    CHECK (
        (deleted_at IS NULL AND ciphertext IS NOT NULL
            AND octet_length(ciphertext) BETWEEN 16 AND 1048576)
        OR
        (deleted_at IS NOT NULL AND ciphertext IS NULL)
    ),
    CHECK (updated_at >= created_at),
    CHECK (deleted_at IS NULL OR deleted_at >= created_at)
);

CREATE INDEX account_model_secrets_account_updated_idx
    ON account_model_secrets (account_id, updated_at DESC, model_id);
