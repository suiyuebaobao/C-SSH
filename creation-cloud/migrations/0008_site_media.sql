CREATE TABLE site_media (
    id UUID PRIMARY KEY,
    slot TEXT NOT NULL CHECK (slot IN ('home_qr')),
    state TEXT NOT NULL DEFAULT 'draft'
        CHECK (state IN ('draft', 'published', 'revoked')),
    storage_key TEXT NOT NULL UNIQUE CHECK (
        storage_key ~ '^objects/[0-9a-f]{2}/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\.png$'
        AND substring(storage_key FROM 9 FOR 2) = substring(storage_key FROM 12 FOR 2)
    ),
    content_type TEXT NOT NULL CHECK (content_type = 'image/png'),
    byte_size BIGINT NOT NULL CHECK (byte_size BETWEEN 1 AND 2097152),
    sha256 TEXT NOT NULL CHECK (sha256 ~ '^[0-9a-f]{64}$'),
    width INTEGER NOT NULL CHECK (width BETWEEN 128 AND 2048),
    height INTEGER NOT NULL CHECK (height BETWEEN 128 AND 2048),
    alt_zh TEXT NOT NULL CHECK (
        length(btrim(alt_zh)) BETWEEN 1 AND 200 AND alt_zh !~ '[[:cntrl:]]'
    ),
    alt_en TEXT NOT NULL CHECK (
        length(btrim(alt_en)) BETWEEN 1 AND 200 AND alt_en !~ '[[:cntrl:]]'
    ),
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    published_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (width = height),
    CHECK (updated_at >= created_at),
    CHECK (published_at IS NULL OR published_at >= created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= published_at),
    CHECK (
        (state = 'draft' AND published_at IS NULL AND revoked_at IS NULL)
        OR (state = 'published' AND published_at IS NOT NULL AND revoked_at IS NULL)
        OR (state = 'revoked' AND published_at IS NOT NULL AND revoked_at IS NOT NULL)
    )
);

CREATE UNIQUE INDEX site_media_one_published_slot_idx
    ON site_media(slot)
    WHERE state = 'published';

CREATE INDEX site_media_history_idx
    ON site_media(slot, created_at DESC, id DESC);

CREATE FUNCTION guard_site_media_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.state <> 'draft' THEN
            RAISE EXCEPTION 'only draft site media can be deleted';
        END IF;
        RETURN OLD;
    END IF;

    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.slot IS DISTINCT FROM OLD.slot
        OR NEW.storage_key IS DISTINCT FROM OLD.storage_key
        OR NEW.content_type IS DISTINCT FROM OLD.content_type
        OR NEW.byte_size IS DISTINCT FROM OLD.byte_size
        OR NEW.sha256 IS DISTINCT FROM OLD.sha256
        OR NEW.width IS DISTINCT FROM OLD.width
        OR NEW.height IS DISTINCT FROM OLD.height
        OR NEW.created_by IS DISTINCT FROM OLD.created_by
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'site media identity is immutable';
    END IF;

    IF OLD.state = 'draft' AND NEW.state NOT IN ('draft', 'published') THEN
        RAISE EXCEPTION 'invalid draft site media transition';
    ELSIF OLD.state = 'published' AND NEW.state NOT IN ('published', 'revoked') THEN
        RAISE EXCEPTION 'invalid published site media transition';
    ELSIF OLD.state = 'revoked' AND NEW IS DISTINCT FROM OLD THEN
        RAISE EXCEPTION 'revoked site media is immutable';
    END IF;

    IF OLD.state <> 'draft' AND (
        NEW.alt_zh IS DISTINCT FROM OLD.alt_zh
        OR NEW.alt_en IS DISTINCT FROM OLD.alt_en
    ) THEN
        RAISE EXCEPTION 'published site media metadata is immutable';
    END IF;

    IF OLD.published_at IS NOT NULL
        AND NEW.published_at IS DISTINCT FROM OLD.published_at THEN
        RAISE EXCEPTION 'site media publication time is immutable';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_media_guard
BEFORE UPDATE OR DELETE ON site_media
FOR EACH ROW EXECUTE FUNCTION guard_site_media_mutation();
