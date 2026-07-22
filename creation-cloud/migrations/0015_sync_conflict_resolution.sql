ALTER TABLE sync_records
    ADD COLUMN source_device_id UUID,
    ADD CONSTRAINT sync_records_source_device_fkey
        FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id)
        ON DELETE CASCADE;

ALTER TABLE sync_mutations
    ADD COLUMN source_device_id UUID,
    ADD CONSTRAINT sync_mutations_source_device_fkey
        FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id)
        ON DELETE CASCADE;

ALTER TABLE sync_conflicts
    ADD COLUMN source_device_id UUID,
    ADD COLUMN resolution TEXT,
    ADD COLUMN resolution_mutation_id UUID,
    ADD COLUMN resolution_hash TEXT,
    ADD COLUMN resolved_revision BIGINT,
    ADD COLUMN resolved_by_device_id UUID,
    ADD CONSTRAINT sync_conflicts_source_device_fkey
        FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id)
        ON DELETE CASCADE,
    ADD CONSTRAINT sync_conflicts_resolved_device_fkey
        FOREIGN KEY (account_id, resolved_by_device_id)
        REFERENCES devices(account_id, id)
        ON DELETE CASCADE,
    ADD CONSTRAINT sync_conflicts_resolution_kind_check
        CHECK (resolution IS NULL OR resolution IN ('keep_remote', 'apply_changes')),
    ADD CONSTRAINT sync_conflicts_resolution_hash_check
        CHECK (resolution_hash IS NULL OR length(resolution_hash) = 64),
    ADD CONSTRAINT sync_conflicts_resolved_revision_check
        CHECK (resolved_revision IS NULL OR resolved_revision >= 0),
    ADD CONSTRAINT sync_conflicts_resolution_state_check
        CHECK (
            (
                resolved_at IS NULL
                AND resolution IS NULL
                AND resolution_mutation_id IS NULL
                AND resolution_hash IS NULL
                AND resolved_revision IS NULL
                AND resolved_by_device_id IS NULL
            )
            OR
            (
                resolved_at IS NOT NULL
                AND resolution IS NOT NULL
                AND resolution_mutation_id IS NOT NULL
                AND resolution_hash IS NOT NULL
                AND resolved_revision IS NOT NULL
                AND resolved_by_device_id IS NOT NULL
            )
        );

CREATE UNIQUE INDEX sync_conflicts_resolution_mutation_idx
    ON sync_conflicts(account_id, resolution_mutation_id)
    WHERE resolution_mutation_id IS NOT NULL;
