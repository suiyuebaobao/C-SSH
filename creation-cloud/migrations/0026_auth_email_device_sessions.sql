ALTER TABLE accounts
    ALTER COLUMN email DROP NOT NULL;

ALTER TABLE accounts
    DROP CONSTRAINT accounts_status_check;

ALTER TABLE accounts
    ADD COLUMN email_verified_at TIMESTAMPTZ,
    ADD COLUMN credential_version BIGINT NOT NULL DEFAULT 1
        CHECK (credential_version > 0),
    ADD CONSTRAINT accounts_status_check
        CHECK (status IN ('pending_verification', 'active', 'disabled'));

UPDATE accounts
SET email_verified_at = COALESCE(email_verified_at, created_at)
WHERE email IS NOT NULL;

ALTER TABLE accounts
    ADD CONSTRAINT accounts_email_role_check
        CHECK (
            email IS NOT NULL
            OR (
                role = 'admin'
                AND admin_login_name IS NOT NULL
            )
        ),
    ADD CONSTRAINT accounts_pending_verification_check
        CHECK (
            status <> 'pending_verification'
            OR (
                role = 'user'
                AND email IS NOT NULL
                AND email_verified_at IS NULL
            )
        ),
    ADD CONSTRAINT accounts_active_user_verified_check
        CHECK (
            status <> 'active'
            OR role <> 'user'
            OR (
                email IS NOT NULL
                AND email_verified_at IS NOT NULL
            )
        ),
    ADD CONSTRAINT accounts_verified_email_check
        CHECK (email_verified_at IS NULL OR email IS NOT NULL);

ALTER TABLE sessions
    ADD COLUMN credential_version BIGINT,
    ADD COLUMN session_kind TEXT NOT NULL DEFAULT 'unbound'
        CHECK (session_kind IN ('unbound', 'device')),
    ADD COLUMN absolute_expires_at TIMESTAMPTZ,
    ADD COLUMN rotated_from_id UUID REFERENCES sessions(id) ON DELETE SET NULL,
    ADD COLUMN revoked_at TIMESTAMPTZ;

UPDATE sessions AS session
SET credential_version = account.credential_version,
    absolute_expires_at = session.expires_at,
    session_kind = CASE
        WHEN session.device_id IS NULL THEN 'unbound'
        ELSE 'device'
    END
FROM accounts AS account
WHERE account.id = session.account_id;

ALTER TABLE sessions
    ALTER COLUMN credential_version SET NOT NULL,
    ALTER COLUMN absolute_expires_at SET NOT NULL,
    ADD CONSTRAINT sessions_expiry_order_check
        CHECK (
            expires_at <= absolute_expires_at
            AND created_at <= absolute_expires_at
        ),
    ADD CONSTRAINT sessions_kind_device_check
        CHECK (
            (session_kind = 'unbound' AND device_id IS NULL)
            OR (session_kind = 'device' AND device_id IS NOT NULL)
        );

CREATE INDEX sessions_active_token_idx
    ON sessions(token_hash)
    WHERE revoked_at IS NULL;

CREATE INDEX sessions_credential_version_idx
    ON sessions(account_id, credential_version)
    WHERE revoked_at IS NULL;

CREATE INDEX sessions_revoked_cleanup_idx
    ON sessions(revoked_at, id)
    WHERE revoked_at IS NOT NULL;

CREATE TABLE email_verification_challenges (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
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

CREATE UNIQUE INDEX email_verification_one_open_idx
    ON email_verification_challenges(account_id)
    WHERE consumed_at IS NULL;

CREATE INDEX email_verification_lookup_idx
    ON email_verification_challenges(email, created_at DESC)
    WHERE consumed_at IS NULL;

CREATE INDEX email_verification_cleanup_idx
    ON email_verification_challenges(expires_at, id);
