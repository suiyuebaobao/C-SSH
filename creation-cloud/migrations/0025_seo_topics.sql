CREATE TABLE seo_topics (
    id UUID PRIMARY KEY,
    locale TEXT NOT NULL CHECK (locale IN ('zh-CN', 'en')),
    phrase TEXT NOT NULL CHECK (
        phrase = btrim(phrase)
        AND char_length(phrase) BETWEEN 2 AND 48
        AND phrase !~ '[[:cntrl:]]'
    ),
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (updated_at >= created_at)
);

CREATE UNIQUE INDEX seo_topics_locale_phrase_ci_idx
    ON seo_topics(locale, lower(phrase));

CREATE INDEX seo_topics_public_order_idx
    ON seo_topics(locale, sort_order, created_at, id)
    WHERE enabled;

CREATE FUNCTION guard_seo_topic_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.created_by IS DISTINCT FROM OLD.created_by
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'seo topic identity is immutable';
    END IF;
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION enforce_seo_topic_enabled_limit() RETURNS TRIGGER AS $$
DECLARE
    enabled_total BIGINT;
BEGIN
    PERFORM pg_advisory_xact_lock(
        hashtextextended('creation-cloud:seo-topics', 0)
    );
    IF NEW.enabled THEN
        SELECT count(*)
        INTO enabled_total
        FROM seo_topics
        WHERE locale = NEW.locale
          AND enabled
          AND id <> NEW.id;
        IF enabled_total >= 12 THEN
            RAISE EXCEPTION 'each locale supports at most 12 enabled seo topics';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER seo_topics_guard
BEFORE UPDATE ON seo_topics
FOR EACH ROW EXECUTE FUNCTION guard_seo_topic_mutation();

CREATE TRIGGER seo_topics_enabled_limit
BEFORE INSERT OR UPDATE OF locale, enabled ON seo_topics
FOR EACH ROW EXECUTE FUNCTION enforce_seo_topic_enabled_limit();
