ALTER TABLE devices
    ADD COLUMN active_session_reference_id UUID
    GENERATED ALWAYS AS (
        CASE WHEN revoked_at IS NULL THEN id ELSE NULL END
    ) STORED;

ALTER TABLE devices
    ADD CONSTRAINT devices_account_id_id_key UNIQUE (account_id, id),
    ADD CONSTRAINT devices_active_session_reference_key
        UNIQUE (account_id, active_session_reference_id);

ALTER TABLE sessions
    ADD COLUMN device_id UUID,
    ADD CONSTRAINT sessions_active_device_fkey
        FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, active_session_reference_id)
        ON UPDATE RESTRICT
        ON DELETE CASCADE;

CREATE INDEX sessions_bound_device_idx
    ON sessions(account_id, device_id)
    WHERE device_id IS NOT NULL;
