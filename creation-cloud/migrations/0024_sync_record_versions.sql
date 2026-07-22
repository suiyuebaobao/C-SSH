-- 先按同步写入的 account -> device -> record 顺序排空现有写事务，再关闭回填与触发器之间的窗口。
-- EXCLUSIVE 仍允许普通只读查询，但会让新的 FOR UPDATE/写事务等待到迁移提交，避免父行锁与源表锁形成死锁。
LOCK TABLE accounts IN EXCLUSIVE MODE;
LOCK TABLE devices IN EXCLUSIVE MODE;
LOCK TABLE sync_records IN SHARE ROW EXCLUSIVE MODE;

CREATE TABLE sync_record_versions (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    revision BIGINT NOT NULL CHECK (revision > 0),
    namespace TEXT NOT NULL,
    record_key TEXT NOT NULL,
    value JSONB,
    is_deleted BOOLEAN NOT NULL,
    source_device_id UUID,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, revision),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE CASCADE,
    CHECK ((is_deleted AND value IS NULL) OR (NOT is_deleted AND value IS NOT NULL))
);

CREATE INDEX sync_record_versions_snapshot_idx
    ON sync_record_versions(account_id, namespace, record_key, revision DESC);

CREATE INDEX sync_record_versions_retention_idx
    ON sync_record_versions(recorded_at, account_id, revision);

INSERT INTO sync_record_versions (
    account_id, revision, namespace, record_key, value,
    is_deleted, source_device_id, recorded_at
)
SELECT account_id, revision, namespace, record_key, value,
       is_deleted, source_device_id, updated_at
FROM sync_records;

CREATE FUNCTION append_sync_record_version() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO sync_record_versions (
        account_id, revision, namespace, record_key, value,
        is_deleted, source_device_id, recorded_at
    ) VALUES (
        NEW.account_id, NEW.revision, NEW.namespace, NEW.record_key, NEW.value,
        NEW.is_deleted, NEW.source_device_id, NEW.updated_at
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER sync_records_insert_version_history
AFTER INSERT ON sync_records
FOR EACH ROW EXECUTE FUNCTION append_sync_record_version();

CREATE TRIGGER sync_records_update_version_history
AFTER UPDATE OF revision ON sync_records
FOR EACH ROW
WHEN (OLD.revision IS DISTINCT FROM NEW.revision)
EXECUTE FUNCTION append_sync_record_version();
