CREATE OR REPLACE FUNCTION guard_release_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'draft' THEN
            RAISE EXCEPTION 'only draft releases can be deleted';
        END IF;
        RETURN OLD;
    END IF;

    IF OLD.status = 'draft' AND NEW.status NOT IN ('draft', 'validating') THEN
        RAISE EXCEPTION 'invalid release status transition';
    ELSIF OLD.status = 'validating' AND NEW.status NOT IN ('validating', 'published') THEN
        RAISE EXCEPTION 'invalid release status transition';
    ELSIF OLD.status = 'published' AND NEW.status NOT IN ('published', 'revoked', 'hidden') THEN
        RAISE EXCEPTION 'invalid release status transition';
    ELSIF OLD.status IN ('revoked', 'hidden') AND NEW.status <> OLD.status THEN
        RAISE EXCEPTION 'terminal release status cannot change';
    END IF;

    IF OLD.status = 'validating' AND NEW.status = 'published' THEN
        IF NOT EXISTS (
            SELECT 1 FROM release_assets WHERE release_id = NEW.id
        ) THEN
            RAISE EXCEPTION 'release requires at least one asset';
        END IF;
        IF EXISTS (
            SELECT 1
            FROM release_assets AS asset
            WHERE asset.release_id = NEW.id
              AND NOT EXISTS (
                  SELECT 1
                  FROM release_sources AS source
                  WHERE source.asset_id = asset.id AND source.enabled
              )
        ) THEN
            RAISE EXCEPTION 'every release asset requires an enabled source';
        END IF;
    END IF;

    IF OLD.status IN ('published', 'revoked', 'hidden') AND (
        NEW.version IS DISTINCT FROM OLD.version
        OR NEW.channel IS DISTINCT FROM OLD.channel
        OR NEW.title_zh IS DISTINCT FROM OLD.title_zh
        OR NEW.title_en IS DISTINCT FROM OLD.title_en
        OR NEW.notes_zh IS DISTINCT FROM OLD.notes_zh
        OR NEW.notes_en IS DISTINCT FROM OLD.notes_en
        OR NEW.published_at IS DISTINCT FROM OLD.published_at
    ) THEN
        RAISE EXCEPTION 'published release metadata is immutable';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION guard_release_asset_mutation() RETURNS TRIGGER AS $$
DECLARE
    release_id_to_check UUID;
    release_status TEXT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        release_id_to_check := OLD.release_id;
    ELSE
        release_id_to_check := NEW.release_id;
    END IF;

    SELECT status INTO release_status
    FROM releases
    WHERE id = release_id_to_check
    FOR UPDATE;

    IF release_status NOT IN ('draft', 'validating') THEN
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

CREATE OR REPLACE FUNCTION guard_release_source_mutation() RETURNS TRIGGER AS $$
DECLARE
    asset_id_to_check UUID;
    release_status TEXT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        asset_id_to_check := OLD.asset_id;
    ELSE
        asset_id_to_check := NEW.asset_id;
    END IF;

    SELECT releases.status INTO release_status
    FROM releases
    JOIN release_assets ON release_assets.release_id = releases.id
    WHERE release_assets.id = asset_id_to_check
    FOR UPDATE OF releases;

    IF TG_OP = 'INSERT' AND release_status IN ('revoked', 'hidden') THEN
        RAISE EXCEPTION 'terminal release cannot receive new sources';
    ELSIF TG_OP = 'DELETE' AND release_status NOT IN ('draft', 'validating') THEN
        RAISE EXCEPTION 'published release sources must be retained';
    ELSIF TG_OP = 'UPDATE' AND release_status NOT IN ('draft', 'validating') AND (
        NEW.asset_id IS DISTINCT FROM OLD.asset_id
        OR NEW.source_kind IS DISTINCT FROM OLD.source_kind
        OR NEW.provider_name IS DISTINCT FROM OLD.provider_name
        OR NEW.local_path IS DISTINCT FROM OLD.local_path
        OR NEW.external_url IS DISTINCT FROM OLD.external_url
    ) THEN
        RAISE EXCEPTION 'published source identity is immutable';
    END IF;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
