-- updater_signature 只属于 Windows EXE/MSI/ZIP。迁移由 sqlx 在单一事务内执行；
-- ACCESS EXCLUSIVE 锁配合现有资产触发器的单字段修复分支，禁止旁路保护。
LOCK TABLE release_assets IN ACCESS EXCLUSIVE MODE;

CREATE OR REPLACE FUNCTION guard_release_asset_mutation() RETURNS TRIGGER AS $$
DECLARE
    release_id_to_check UUID;
    release_status TEXT;
    records_installed_identity BOOLEAN := FALSE;
    records_windows_signature BOOLEAN := FALSE;
    cleans_non_windows_signature BOOLEAN := FALSE;
BEGIN
    IF TG_OP = 'DELETE' THEN
        release_id_to_check := OLD.release_id;
    ELSE
        release_id_to_check := NEW.release_id;
    END IF;

    IF TG_OP = 'INSERT' AND NEW.installed_sha256 IS NOT NULL THEN
        RAISE EXCEPTION 'installed identity evidence must be recorded separately';
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW.installed_sha256 IS DISTINCT FROM OLD.installed_sha256 THEN
            IF OLD.installed_sha256 IS NOT NULL OR NEW.installed_sha256 IS NULL THEN
                RAISE EXCEPTION 'installed identity evidence is immutable';
            END IF;
            records_installed_identity := TRUE;
        END IF;

        IF NEW.updater_signature IS DISTINCT FROM OLD.updater_signature
            AND OLD.updater_signature IS NULL
            AND NEW.updater_signature IS NOT NULL
            AND OLD.platform = 'windows'
            AND OLD.package_kind IN ('exe', 'msi', 'zip') THEN
            records_windows_signature := TRUE;
        END IF;

        cleans_non_windows_signature :=
            OLD.updater_signature IS NOT NULL
            AND NEW.updater_signature IS NULL
            AND NOT (
                OLD.platform = 'windows'
                AND OLD.package_kind IN ('exe', 'msi', 'zip')
            )
            AND NEW.id IS NOT DISTINCT FROM OLD.id
            AND NEW.release_id IS NOT DISTINCT FROM OLD.release_id
            AND NEW.platform IS NOT DISTINCT FROM OLD.platform
            AND NEW.architecture IS NOT DISTINCT FROM OLD.architecture
            AND NEW.package_kind IS NOT DISTINCT FROM OLD.package_kind
            AND NEW.file_name IS NOT DISTINCT FROM OLD.file_name
            AND NEW.byte_size IS NOT DISTINCT FROM OLD.byte_size
            AND NEW.sha256 IS NOT DISTINCT FROM OLD.sha256
            AND NEW.installed_sha256 IS NOT DISTINCT FROM OLD.installed_sha256
            AND NEW.created_at IS NOT DISTINCT FROM OLD.created_at;

        IF (records_installed_identity OR records_windows_signature) AND (
            NEW.id IS DISTINCT FROM OLD.id
            OR NEW.release_id IS DISTINCT FROM OLD.release_id
            OR NEW.platform IS DISTINCT FROM OLD.platform
            OR NEW.architecture IS DISTINCT FROM OLD.architecture
            OR NEW.package_kind IS DISTINCT FROM OLD.package_kind
            OR NEW.file_name IS DISTINCT FROM OLD.file_name
            OR NEW.byte_size IS DISTINCT FROM OLD.byte_size
            OR NEW.sha256 IS DISTINCT FROM OLD.sha256
            OR (
                records_installed_identity
                AND NEW.updater_signature IS DISTINCT FROM OLD.updater_signature
            )
            OR (
                records_windows_signature
                AND NEW.installed_sha256 IS DISTINCT FROM OLD.installed_sha256
            )
            OR NEW.created_at IS DISTINCT FROM OLD.created_at
        ) THEN
            RAISE EXCEPTION 'installed identity evidence must be recorded separately';
        END IF;

        IF OLD.installed_sha256 IS NOT NULL AND (
            NEW.release_id IS DISTINCT FROM OLD.release_id
            OR NEW.platform IS DISTINCT FROM OLD.platform
            OR NEW.architecture IS DISTINCT FROM OLD.architecture
            OR NEW.package_kind IS DISTINCT FROM OLD.package_kind
            OR NEW.file_name IS DISTINCT FROM OLD.file_name
            OR NEW.byte_size IS DISTINCT FROM OLD.byte_size
            OR NEW.sha256 IS DISTINCT FROM OLD.sha256
        ) THEN
            RAISE EXCEPTION 'asset identity with installed evidence is immutable';
        END IF;
    END IF;

    SELECT status INTO release_status
    FROM releases
    WHERE id = release_id_to_check
    FOR UPDATE;

    IF release_status NOT IN ('draft', 'validating')
        AND NOT records_installed_identity
        AND NOT records_windows_signature
        AND NOT cleans_non_windows_signature THEN
        RAISE EXCEPTION 'assets of a published release are immutable';
    END IF;

    IF TG_OP = 'UPDATE' AND (
        NEW.byte_size IS DISTINCT FROM OLD.byte_size
        OR NEW.sha256 IS DISTINCT FROM OLD.sha256
    ) AND EXISTS (
        SELECT 1
        FROM release_sources
        WHERE asset_id = OLD.id AND source_kind = 'local'
    ) THEN
        RAISE EXCEPTION 'local source asset identity is immutable';
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

UPDATE release_assets
SET updater_signature = NULL
WHERE updater_signature IS NOT NULL
  AND NOT (
      platform = 'windows'
      AND package_kind IN ('exe', 'msi', 'zip')
  );

ALTER TABLE release_assets
ADD CONSTRAINT release_assets_updater_signature_scope_check
CHECK (
    updater_signature IS NULL
    OR (
        platform = 'windows'
        AND package_kind IN ('exe', 'msi', 'zip')
    )
);

-- 发布前保留服务层之外的数据库门禁；缺失与误带两种方向都失败关闭。
CREATE OR REPLACE FUNCTION guard_release_signature_publication() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'validating' AND NEW.status = 'published' THEN
        IF EXISTS (
            SELECT 1 FROM release_assets AS asset
            WHERE asset.release_id = NEW.id
              AND asset.platform = 'windows'
              AND asset.package_kind IN ('exe', 'msi', 'zip')
              AND asset.updater_signature IS NULL
        ) THEN
            RAISE EXCEPTION 'windows update assets require updater signatures';
        END IF;
        IF EXISTS (
            SELECT 1 FROM release_assets AS asset
            WHERE asset.release_id = NEW.id
              AND NOT (
                  asset.platform = 'windows'
                  AND asset.package_kind IN ('exe', 'msi', 'zip')
              )
              AND asset.updater_signature IS NOT NULL
        ) THEN
            RAISE EXCEPTION 'non-windows assets cannot carry updater signatures';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
