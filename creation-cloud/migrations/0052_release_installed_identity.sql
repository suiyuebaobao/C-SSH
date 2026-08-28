-- 当前安装身份摘要独立于下载资产摘要；旧 Windows 资产保持 NULL，等待受控编排回填。
ALTER TABLE release_assets
ADD COLUMN installed_sha256 TEXT
CHECK (
    installed_sha256 IS NULL
    OR installed_sha256 ~ '^[0-9a-f]{64}$'
);

-- Portable ZIP 与两种安装器统一要求 updater metadata；Android 仍由 APK 平台签名链负责。
CREATE OR REPLACE FUNCTION guard_release_signature_publication() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'validating' AND NEW.status = 'published' AND EXISTS (
        SELECT 1 FROM release_assets AS asset
        WHERE asset.release_id = NEW.id
          AND asset.platform = 'windows'
          AND asset.package_kind IN ('exe', 'msi', 'zip')
          AND asset.updater_signature IS NULL
    ) THEN
        RAISE EXCEPTION 'windows update assets require updater signatures';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 证据只能从空值写入一次；写入后资产身份不可再改变，避免摘要与资产元数据脱钩。
CREATE OR REPLACE FUNCTION guard_release_asset_mutation() RETURNS TRIGGER AS $$
DECLARE
    release_id_to_check UUID;
    release_status TEXT;
    records_installed_identity BOOLEAN := FALSE;
    records_windows_signature BOOLEAN := FALSE;
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
        AND NOT records_windows_signature THEN
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

-- Android 单一 base APK 的安装身份就是最终 APK 摘要，可在迁移中确定性安全回填。
UPDATE release_assets
SET installed_sha256 = sha256
WHERE platform = 'android'
  AND architecture = 'aarch64'
  AND package_kind = 'apk'
  AND installed_sha256 IS NULL;
