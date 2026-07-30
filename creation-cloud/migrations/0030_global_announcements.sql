CREATE TABLE global_announcements (
    id UUID PRIMARY KEY,
    title_zh_cn TEXT NOT NULL
        CHECK (char_length(title_zh_cn) BETWEEN 1 AND 160),
    body_zh_cn TEXT NOT NULL
        CHECK (char_length(body_zh_cn) BETWEEN 1 AND 10000),
    title_en TEXT NOT NULL
        CHECK (char_length(title_en) BETWEEN 1 AND 160),
    body_en TEXT NOT NULL
        CHECK (char_length(body_en) BETWEEN 1 AND 10000),
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published', 'hidden')),
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision >= 1),
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    published_at TIMESTAMPTZ,
    hidden_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (updated_at >= created_at),
    CHECK (
        (status = 'draft' AND published_at IS NULL AND hidden_at IS NULL)
        OR (status = 'published' AND published_at IS NOT NULL AND hidden_at IS NULL)
        OR (status = 'hidden' AND published_at IS NOT NULL AND hidden_at IS NOT NULL)
    ),
    CHECK (published_at IS NULL OR published_at >= created_at),
    CHECK (hidden_at IS NULL OR hidden_at >= published_at)
);

CREATE UNIQUE INDEX global_announcements_one_published_idx
    ON global_announcements ((1))
    WHERE status = 'published';

CREATE INDEX global_announcements_admin_order_idx
    ON global_announcements (created_at DESC, id DESC);

CREATE INDEX global_announcements_current_idx
    ON global_announcements (published_at DESC, id DESC)
    WHERE status = 'published';

CREATE TABLE global_announcement_publication_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    public_revision BIGINT NOT NULL DEFAULT 0 CHECK (public_revision >= 0),
    current_announcement_id UUID
        REFERENCES global_announcements(id) ON DELETE RESTRICT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO global_announcement_publication_state (
    singleton, public_revision, current_announcement_id
) VALUES (TRUE, 0, NULL);

CREATE FUNCTION guard_global_announcement_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'draft' THEN
            RAISE EXCEPTION 'only draft announcements can be deleted';
        END IF;
        RETURN OLD;
    END IF;

    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.created_by IS DISTINCT FROM OLD.created_by
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'announcement identity is immutable';
    END IF;

    IF NEW.revision <> OLD.revision + 1
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid announcement revision';
    END IF;

    IF OLD.status = 'draft' AND NEW.status = 'draft' THEN
        IF NEW.published_at IS NOT NULL OR NEW.hidden_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid announcement draft update';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.status = 'draft' AND NEW.status = 'published' THEN
        IF NEW.title_zh_cn IS DISTINCT FROM OLD.title_zh_cn
            OR NEW.body_zh_cn IS DISTINCT FROM OLD.body_zh_cn
            OR NEW.title_en IS DISTINCT FROM OLD.title_en
            OR NEW.body_en IS DISTINCT FROM OLD.body_en
            OR NEW.published_at IS NULL
            OR NEW.hidden_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid announcement publication';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.status = 'published' AND NEW.status = 'hidden' THEN
        IF NEW.title_zh_cn IS DISTINCT FROM OLD.title_zh_cn
            OR NEW.body_zh_cn IS DISTINCT FROM OLD.body_zh_cn
            OR NEW.title_en IS DISTINCT FROM OLD.title_en
            OR NEW.body_en IS DISTINCT FROM OLD.body_en
            OR NEW.published_at IS DISTINCT FROM OLD.published_at
            OR NEW.hidden_at IS NULL THEN
            RAISE EXCEPTION 'invalid announcement hide';
        END IF;
        RETURN NEW;
    END IF;

    RAISE EXCEPTION 'invalid announcement state transition';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER global_announcements_guard
BEFORE UPDATE OR DELETE ON global_announcements
FOR EACH ROW EXECUTE FUNCTION guard_global_announcement_mutation();

CREATE FUNCTION guard_global_announcement_publication_state() RETURNS TRIGGER AS $$
DECLARE
    published_count BIGINT;
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'global announcement publication state cannot be deleted';
    END IF;

    IF NEW.singleton IS DISTINCT FROM OLD.singleton
        OR NEW.public_revision <> OLD.public_revision + 1
        OR NEW.current_announcement_id IS NOT DISTINCT FROM OLD.current_announcement_id
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid global announcement publication revision';
    END IF;

    SELECT count(*) INTO published_count
    FROM global_announcements
    WHERE status = 'published';

    IF NEW.current_announcement_id IS NULL THEN
        IF published_count <> 0 THEN
            RAISE EXCEPTION 'publication state does not match published announcement';
        END IF;
    ELSIF published_count <> 1 OR NOT EXISTS (
        SELECT 1
        FROM global_announcements
        WHERE id = NEW.current_announcement_id
          AND status = 'published'
    ) THEN
        RAISE EXCEPTION 'publication state does not match published announcement';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER global_announcement_publication_state_guard
BEFORE UPDATE OR DELETE ON global_announcement_publication_state
FOR EACH ROW EXECUTE FUNCTION guard_global_announcement_publication_state();

CREATE FUNCTION check_global_announcement_publication_consistency() RETURNS TRIGGER AS $$
DECLARE
    state_current UUID;
    published_count BIGINT;
    published_id UUID;
BEGIN
    SELECT current_announcement_id INTO state_current
    FROM global_announcement_publication_state
    WHERE singleton = TRUE;

    SELECT count(*) INTO published_count
    FROM global_announcements
    WHERE status = 'published';

    SELECT id INTO published_id
    FROM global_announcements
    WHERE status = 'published'
    LIMIT 1;

    IF (published_count = 0 AND state_current IS NULL)
        OR (
            published_count = 1
            AND state_current IS NOT DISTINCT FROM published_id
        ) THEN
        RETURN NULL;
    END IF;

    RAISE EXCEPTION 'published announcement and publication state are inconsistent';
END;
$$ LANGUAGE plpgsql;

CREATE CONSTRAINT TRIGGER global_announcement_publication_consistency
AFTER INSERT OR UPDATE OR DELETE ON global_announcements
DEFERRABLE INITIALLY DEFERRED
FOR EACH ROW EXECUTE FUNCTION check_global_announcement_publication_consistency();
