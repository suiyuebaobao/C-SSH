CREATE TABLE admin_login_captcha_challenges (
    id UUID PRIMARY KEY,
    code_digest BYTEA NOT NULL CHECK (octet_length(code_digest) = 32),
    attempt_count INTEGER NOT NULL DEFAULT 0
        CHECK (attempt_count BETWEEN 0 AND 5),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at > created_at),
    CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);

CREATE INDEX admin_login_captcha_open_idx
    ON admin_login_captcha_challenges(expires_at, id)
    WHERE consumed_at IS NULL;
