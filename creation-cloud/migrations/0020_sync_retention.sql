CREATE TABLE sync_device_checkpoints (
    account_id UUID NOT NULL,
    device_id UUID NOT NULL,
    acknowledged_revision BIGINT NOT NULL DEFAULT 0 CHECK (acknowledged_revision >= 0),
    last_sync_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, device_id),
    FOREIGN KEY (account_id, device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE
);

CREATE INDEX sync_device_checkpoints_active_idx
    ON sync_device_checkpoints(account_id, last_sync_at, acknowledged_revision, device_id);

ALTER TABLE sync_states
    ADD COLUMN compacted_through_revision BIGINT NOT NULL DEFAULT 0,
    ADD CONSTRAINT sync_states_compaction_floor_check
        CHECK (
            compacted_through_revision >= 0
            AND compacted_through_revision <= current_revision
        );

CREATE INDEX sync_records_tombstone_retention_idx
    ON sync_records(account_id, updated_at, revision, id)
    WHERE is_deleted = TRUE;

CREATE INDEX sync_mutations_applied_retention_idx
    ON sync_mutations(created_at, account_id, client_mutation_id)
    WHERE outcome = 'applied';

CREATE INDEX sync_conflicts_resolved_retention_idx
    ON sync_conflicts(resolved_at, account_id, id)
    WHERE resolved_at IS NOT NULL;
