-- 官方代理只保存节点目录和不可逆摘要；不保存目标主机、目标端口或短凭据明文。
CREATE TABLE proxy_nodes (
    id UUID PRIMARY KEY,
    display_name TEXT NOT NULL
        CHECK (char_length(display_name) BETWEEN 1 AND 80)
        CHECK (display_name !~ '[[:cntrl:]]'),
    region TEXT NOT NULL
        CHECK (region ~ '^[a-z0-9][a-z0-9-]{0,31}$'),
    environment TEXT NOT NULL
        CHECK (environment IN ('test', 'staging', 'production')),
    status TEXT NOT NULL DEFAULT 'disabled'
        CHECK (status IN ('active', 'draining', 'disabled')),
    fail_closed_gateway_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (environment, display_name)
);

CREATE TABLE proxy_node_endpoints (
    id UUID PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES proxy_nodes(id) ON DELETE CASCADE,
    protocol TEXT NOT NULL CHECK (protocol = 'http_connect'),
    host TEXT NOT NULL
        CHECK (char_length(host) BETWEEN 1 AND 253)
        CHECK (host !~ '[[:space:]/?#@]'),
    port INTEGER NOT NULL CHECK (port BETWEEN 1 AND 65535),
    transport_security TEXT NOT NULL CHECK (transport_security = 'tls'),
    tls_server_name TEXT NOT NULL
        CHECK (char_length(tls_server_name) BETWEEN 1 AND 253)
        CHECK (
            tls_server_name ~ '^[A-Za-z0-9](?:[A-Za-z0-9.-]{0,251}[A-Za-z0-9])?$'
        ),
    priority SMALLINT NOT NULL DEFAULT 100 CHECK (priority BETWEEN 0 AND 1000),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (node_id, protocol, host, port)
);

CREATE INDEX proxy_node_endpoints_directory_idx
    ON proxy_node_endpoints(node_id, priority, id)
    WHERE enabled;

-- 节点 bearer 由部署侧生成，数据库只保存 SHA-256 摘要。
CREATE TABLE proxy_node_auth_keys (
    id UUID PRIMARY KEY,
    node_id UUID NOT NULL REFERENCES proxy_nodes(id) ON DELETE CASCADE,
    label TEXT NOT NULL
        CHECK (char_length(label) BETWEEN 1 AND 64)
        CHECK (label !~ '[[:cntrl:]]'),
    token_digest BYTEA NOT NULL UNIQUE CHECK (octet_length(token_digest) = 32),
    not_before TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at IS NULL OR expires_at > not_before),
    CHECK (revoked_at IS NULL OR revoked_at >= created_at)
);

CREATE INDEX proxy_node_auth_keys_active_idx
    ON proxy_node_auth_keys(node_id, not_before, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE proxy_device_credentials (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    session_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    node_id UUID NOT NULL REFERENCES proxy_nodes(id) ON DELETE RESTRICT,
    username TEXT NOT NULL UNIQUE
        CHECK (username ~ '^cssh_[0-9a-f]{32}$'),
    key_version INTEGER NOT NULL CHECK (key_version > 0),
    account_credential_version BIGINT NOT NULL
        CHECK (account_credential_version > 0),
    password_digest BYTEA NOT NULL CHECK (octet_length(password_digest) = 32),
    target_digest BYTEA NOT NULL CHECK (octet_length(target_digest) = 32),
    mutation_id UUID NOT NULL,
    request_digest BYTEA NOT NULL CHECK (octet_length(request_digest) = 32),
    generation BIGINT NOT NULL CHECK (generation > 0),
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    not_before TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    UNIQUE (id, account_id, device_id, node_id),
    UNIQUE (account_id, device_id, mutation_id),
    UNIQUE (account_id, device_id, node_id, target_digest, generation),
    CHECK (not_before >= issued_at),
    CHECK (expires_at > not_before),
    CHECK (expires_at <= issued_at + interval '15 minutes'),
    CHECK (revoked_at IS NULL OR revoked_at >= issued_at)
);

CREATE INDEX proxy_device_credentials_authorize_idx
    ON proxy_device_credentials(node_id, username)
    WHERE revoked_at IS NULL;

CREATE INDEX proxy_device_credentials_rotation_idx
    ON proxy_device_credentials(account_id, device_id, node_id, target_digest, generation DESC);

CREATE INDEX proxy_device_credentials_expiry_idx
    ON proxy_device_credentials(expires_at, id)
    WHERE revoked_at IS NULL;

-- 短凭据只负责新 CONNECT admission；活动隧道只保存 opaque lease 身份，
-- 不持久化目标、密码、流量或来源 IP。
CREATE TABLE proxy_tunnel_leases (
    connection_id UUID PRIMARY KEY,
    credential_id UUID NOT NULL,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    device_id UUID NOT NULL,
    node_id UUID NOT NULL REFERENCES proxy_nodes(id) ON DELETE CASCADE,
    admitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_authorized_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    lease_expires_at TIMESTAMPTZ NOT NULL,
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    FOREIGN KEY (credential_id, account_id, device_id, node_id)
        REFERENCES proxy_device_credentials(id, account_id, device_id, node_id)
        ON DELETE CASCADE,
    CHECK (lease_expires_at > last_authorized_at)
);

CREATE INDEX proxy_tunnel_leases_credential_idx
    ON proxy_tunnel_leases(credential_id, lease_expires_at);

CREATE INDEX proxy_tunnel_leases_expiry_idx
    ON proxy_tunnel_leases(lease_expires_at, connection_id);
