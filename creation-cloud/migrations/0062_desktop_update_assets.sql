-- Add future desktop update assets without rewriting any existing release or metadata.
LOCK TABLE release_assets IN ACCESS EXCLUSIVE MODE;
CREATE FUNCTION desktop_updater_signature_asset(platform TEXT, kind TEXT) RETURNS BOOLEAN
LANGUAGE SQL IMMUTABLE AS $$
    SELECT (platform = 'windows' AND kind IN ('exe','msi','zip'))
        OR (platform = 'macos' AND kind = 'tar_gz')
        OR (platform = 'linux' AND kind = 'appimage');
$$;
CREATE FUNCTION formal_update_asset(release_id UUID, platform TEXT, architecture TEXT, kind TEXT)
RETURNS BOOLEAN LANGUAGE SQL STABLE AS $$
    SELECT (platform = 'windows' AND architecture = 'x86_64' AND kind IN ('exe','msi','zip'))
        OR (platform = 'android' AND architecture = 'aarch64' AND kind = 'apk')
        OR (EXISTS (SELECT 1 FROM release_assets a WHERE a.release_id = $1
                        AND a.platform = 'macos' AND a.package_kind = 'tar_gz')
            AND ((platform = 'macos' AND architecture = 'aarch64' AND kind = 'tar_gz')
                OR (platform = 'linux' AND architecture IN ('x86_64','aarch64') AND kind = 'appimage')));
$$;
ALTER TABLE release_assets DROP CONSTRAINT release_assets_updater_signature_scope_check;
ALTER TABLE release_assets ADD CONSTRAINT release_assets_updater_signature_scope_check
CHECK (updater_signature IS NULL OR desktop_updater_signature_asset(platform, package_kind));

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
        AND TG_OP = 'UPDATE' AND OLD.platform IN ('macos', 'linux')
        AND records_installed_identity THEN
        RAISE EXCEPTION 'published desktop installed identity is immutable';
    END IF;

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

CREATE OR REPLACE FUNCTION guard_release_signature_publication() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'validating' AND NEW.status = 'published' THEN
        IF EXISTS (SELECT 1 FROM release_assets a WHERE a.release_id = NEW.id
            AND desktop_updater_signature_asset(a.platform, a.package_kind)
            AND (a.platform = 'windows' OR formal_update_asset(a.release_id, a.platform, a.architecture, a.package_kind))
            AND a.updater_signature IS NULL) THEN
            RAISE EXCEPTION 'desktop update assets require updater signatures';
        END IF;
        IF EXISTS (SELECT 1 FROM release_assets a WHERE a.release_id = NEW.id
            AND NOT desktop_updater_signature_asset(a.platform, a.package_kind)
            AND a.updater_signature IS NOT NULL) THEN
            RAISE EXCEPTION 'unsupported assets cannot carry updater signatures';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
