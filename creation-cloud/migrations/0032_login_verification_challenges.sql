CREATE TABLE login_verification_challenges (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    credential_version BIGINT NOT NULL CHECK (credential_version > 0),
    code_digest BYTEA NOT NULL CHECK (octet_length(code_digest) = 32),
    attempt_count INTEGER NOT NULL DEFAULT 0
        CHECK (attempt_count BETWEEN 0 AND 5),
    expires_at TIMESTAMPTZ NOT NULL,
    sent_at TIMESTAMPTZ,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at > created_at),
    CHECK (sent_at IS NULL OR sent_at >= created_at),
    CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);

CREATE UNIQUE INDEX login_verification_one_open_idx
    ON login_verification_challenges(account_id)
    WHERE consumed_at IS NULL;

CREATE INDEX login_verification_cleanup_idx
    ON login_verification_challenges(expires_at, id);
