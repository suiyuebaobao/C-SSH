CREATE TABLE site_content_revisions (
    id UUID PRIMARY KEY,
    document_key TEXT NOT NULL
        CHECK (document_key IN ('site_shell', 'home')),
    locale TEXT NOT NULL
        CHECK (locale IN ('zh-CN', 'en')),
    state TEXT NOT NULL
        CHECK (state IN ('draft', 'published', 'revoked')),
    revision BIGINT NOT NULL CHECK (revision >= 1),
    content_json JSONB NOT NULL
        CHECK (
            jsonb_typeof(content_json) = 'object'
            AND octet_length(content_json::TEXT) BETWEEN 2 AND 131072
        ),
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    published_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (updated_at >= created_at),
    CHECK (
        (state = 'draft' AND published_at IS NULL AND revoked_at IS NULL)
        OR (state = 'published' AND published_at IS NOT NULL AND revoked_at IS NULL)
        OR (state = 'revoked' AND published_at IS NOT NULL AND revoked_at IS NOT NULL)
    ),
    CHECK (published_at IS NULL OR published_at >= created_at),
    CHECK (revoked_at IS NULL OR revoked_at >= published_at)
);

CREATE UNIQUE INDEX site_content_one_published_document_locale_idx
    ON site_content_revisions(document_key, locale)
    WHERE state = 'published';

CREATE INDEX site_content_history_idx
    ON site_content_revisions(document_key, locale, created_at DESC, id DESC);

CREATE INDEX site_content_draft_idx
    ON site_content_revisions(document_key, locale, updated_at DESC, id DESC)
    WHERE state = 'draft';

CREATE FUNCTION guard_site_content_revision_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.state <> 'draft' THEN
            RAISE EXCEPTION 'published site content history is immutable';
        END IF;
        RETURN OLD;
    END IF;

    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.document_key IS DISTINCT FROM OLD.document_key
        OR NEW.locale IS DISTINCT FROM OLD.locale
        OR NEW.created_by IS DISTINCT FROM OLD.created_by
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'site content identity is immutable';
    END IF;

    IF OLD.state = 'draft' AND NEW.state = 'draft' THEN
        IF NEW.revision <> OLD.revision + 1
            OR NEW.published_at IS NOT NULL
            OR NEW.revoked_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid site content draft update';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.state = 'draft' AND NEW.state = 'published' THEN
        IF NEW.revision <> OLD.revision
            OR NEW.content_json IS DISTINCT FROM OLD.content_json
            OR NEW.published_at IS NULL
            OR NEW.revoked_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid site content publication';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.state = 'published' AND NEW.state = 'revoked' THEN
        IF NEW.revision <> OLD.revision
            OR NEW.content_json IS DISTINCT FROM OLD.content_json
            OR NEW.published_at IS DISTINCT FROM OLD.published_at
            OR NEW.revoked_at IS NULL THEN
            RAISE EXCEPTION 'invalid site content revocation';
        END IF;
        RETURN NEW;
    END IF;

    RAISE EXCEPTION 'invalid site content state transition';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER site_content_revision_guard
BEFORE UPDATE OR DELETE ON site_content_revisions
FOR EACH ROW EXECUTE FUNCTION guard_site_content_revision_mutation();
