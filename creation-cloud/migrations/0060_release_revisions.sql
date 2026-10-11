-- 同版本修订使用独立发布身份；旧行、资产及来源保持不变。
ALTER TABLE releases ADD COLUMN revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0);
ALTER TABLE releases DROP CONSTRAINT releases_version_key;
ALTER TABLE releases ADD CONSTRAINT releases_version_revision_key UNIQUE (version, revision);

-- 分配记录独立于可删除草稿，已用号码不回收。
CREATE TABLE release_revision_counters (
    version TEXT PRIMARY KEY,
    last_revision INTEGER NOT NULL CHECK (last_revision > 0)
);
INSERT INTO release_revision_counters (version, last_revision)
SELECT version, revision FROM releases;

CREATE FUNCTION guard_release_revision_identity() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.version IS DISTINCT FROM OLD.version
       OR NEW.channel IS DISTINCT FROM OLD.channel
       OR NEW.revision IS DISTINCT FROM OLD.revision THEN
        RAISE EXCEPTION 'release version, channel and revision are immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER releases_revision_identity_guard
BEFORE UPDATE ON releases
FOR EACH ROW EXECUTE FUNCTION guard_release_revision_identity();
