CREATE TABLE releases (
    id UUID PRIMARY KEY,
    version TEXT NOT NULL UNIQUE,
    channel TEXT NOT NULL CHECK (channel IN ('stable', 'beta', 'nightly')),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'validating', 'published', 'revoked', 'hidden')),
    title_zh TEXT NOT NULL,
    title_en TEXT NOT NULL,
    notes_zh TEXT NOT NULL,
    notes_en TEXT NOT NULL,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (status IN ('draft', 'validating') AND published_at IS NULL)
        OR
        (status IN ('published', 'revoked', 'hidden') AND published_at IS NOT NULL)
    )
);

CREATE INDEX releases_public_idx ON releases(status, published_at DESC);

CREATE TABLE release_assets (
    id UUID PRIMARY KEY,
    release_id UUID NOT NULL REFERENCES releases(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    architecture TEXT NOT NULL,
    package_kind TEXT NOT NULL,
    file_name TEXT NOT NULL,
    byte_size BIGINT NOT NULL CHECK (byte_size >= 0),
    sha256 TEXT NOT NULL CHECK (sha256 ~ '^[0-9a-f]{64}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (release_id, platform, architecture, package_kind)
);

CREATE TABLE release_sources (
    id UUID PRIMARY KEY,
    asset_id UUID NOT NULL REFERENCES release_assets(id) ON DELETE CASCADE,
    source_kind TEXT NOT NULL CHECK (source_kind IN ('local', 'external')),
    provider_name TEXT NOT NULL,
    local_path TEXT,
    external_url TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (
            source_kind = 'local'
            AND local_path IS NOT NULL
            AND external_url IS NULL
            AND local_path <> ''
            AND left(local_path, 1) NOT IN ('/', chr(92))
            AND local_path !~ '^[A-Za-z]:'
            AND replace(local_path, chr(92), '/') <> '..'
            AND position('../' IN replace(local_path, chr(92), '/')) = 0
            AND right(replace(local_path, chr(92), '/'), 3) <> '/..'
        )
        OR
        (
            source_kind = 'external'
            AND local_path IS NULL
            AND external_url IS NOT NULL
            AND external_url ~ '^https://[^[:space:]]+$'
            AND external_url !~ '^https://[^/]*@'
        )
    )
);

CREATE INDEX release_sources_asset_id_idx ON release_sources(asset_id);
CREATE INDEX release_sources_public_idx
    ON release_sources(asset_id, enabled, sort_order, created_at);

CREATE TABLE download_events (
    id UUID PRIMARY KEY,
    asset_id UUID NOT NULL REFERENCES release_assets(id) ON DELETE RESTRICT,
    source_id UUID NOT NULL REFERENCES release_sources(id) ON DELETE RESTRICT,
    account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX download_events_occurred_at_idx ON download_events(occurred_at DESC);

CREATE FUNCTION guard_release_mutation() RETURNS TRIGGER AS $$
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

CREATE TRIGGER releases_guard
BEFORE UPDATE OR DELETE ON releases
FOR EACH ROW EXECUTE FUNCTION guard_release_mutation();

CREATE FUNCTION guard_release_asset_mutation() RETURNS TRIGGER AS $$
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
    WHERE id = release_id_to_check;

    IF release_status NOT IN ('draft', 'validating') THEN
        RAISE EXCEPTION 'assets of a published release are immutable';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER release_assets_guard
BEFORE INSERT OR UPDATE OR DELETE ON release_assets
FOR EACH ROW EXECUTE FUNCTION guard_release_asset_mutation();

CREATE FUNCTION guard_release_source_mutation() RETURNS TRIGGER AS $$
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
    FROM release_assets
    JOIN releases ON releases.id = release_assets.release_id
    WHERE release_assets.id = asset_id_to_check;

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

CREATE TRIGGER release_sources_guard
BEFORE INSERT OR UPDATE OR DELETE ON release_sources
FOR EACH ROW EXECUTE FUNCTION guard_release_source_mutation();
