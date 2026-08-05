ALTER TABLE sessions
    ADD COLUMN last_login_ip INET,
    ADD COLUMN user_agent TEXT,
    ADD COLUMN client_version TEXT,
    ADD COLUMN device_fingerprint TEXT,
    ADD CONSTRAINT sessions_user_agent_check
        CHECK (
            user_agent IS NULL
            OR (
                char_length(user_agent) BETWEEN 1 AND 512
                AND user_agent !~ '[[:cntrl:]]'
            )
        ),
    ADD CONSTRAINT sessions_client_version_check
        CHECK (
            client_version IS NULL
            OR (
                char_length(client_version) BETWEEN 1 AND 64
                AND client_version !~ '[[:cntrl:]]'
            )
        ),
    ADD CONSTRAINT sessions_device_fingerprint_check
        CHECK (
            device_fingerprint IS NULL
            OR device_fingerprint ~ '^[0-9a-f]{64}$'
        );

CREATE INDEX sessions_account_activity_idx
    ON sessions(account_id, last_seen_at DESC, id);
