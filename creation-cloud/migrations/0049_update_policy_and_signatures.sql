-- 客户端更新资产的签名元数据由上传流程写入，发布策略不得由管理员手填。
ALTER TABLE release_assets
ADD COLUMN updater_signature TEXT
CHECK (
    updater_signature IS NULL
    OR (
        octet_length(updater_signature) BETWEEN 16 AND 8192
        AND updater_signature !~ '[[:cntrl:]]'
    )
);

CREATE FUNCTION guard_release_signature_publication() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status = 'validating' AND NEW.status = 'published' AND EXISTS (
        SELECT 1 FROM release_assets AS asset
        WHERE asset.release_id = NEW.id
          AND asset.platform = 'windows'
          AND asset.package_kind IN ('exe', 'msi')
          AND asset.updater_signature IS NULL
    ) THEN
        RAISE EXCEPTION 'windows installer assets require updater signatures';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER releases_signature_publish_guard
BEFORE UPDATE OF status ON releases
FOR EACH ROW EXECUTE FUNCTION guard_release_signature_publication();

CREATE TABLE update_policy_draft (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    forced_versions TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[]
        CHECK (cardinality(forced_versions) <= 128),
    target_release_id UUID REFERENCES releases(id) ON DELETE RESTRICT,
    sha256_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    updated_by UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (
        (enabled AND target_release_id IS NOT NULL)
        OR (NOT enabled AND target_release_id IS NULL AND cardinality(forced_versions) = 0)
    )
);

INSERT INTO update_policy_draft (singleton) VALUES (TRUE);

CREATE TABLE update_policy_publications (
    revision BIGINT PRIMARY KEY CHECK (revision > 0),
    enabled BOOLEAN NOT NULL,
    forced_versions TEXT[] NOT NULL CHECK (cardinality(forced_versions) <= 128),
    target_release_id UUID REFERENCES releases(id) ON DELETE RESTRICT,
    sha256_enabled BOOLEAN NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    CHECK (
        (enabled AND target_release_id IS NOT NULL)
        OR (NOT enabled AND target_release_id IS NULL AND cardinality(forced_versions) = 0)
    )
);

CREATE TABLE update_policy_publication_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    current_revision BIGINT REFERENCES update_policy_publications(revision) ON DELETE RESTRICT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO update_policy_publication_state (singleton) VALUES (TRUE);

CREATE FUNCTION guard_update_policy_publication_advance() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.current_revision IS NULL
       OR (OLD.current_revision IS NOT NULL AND NEW.current_revision <= OLD.current_revision) THEN
        RAISE EXCEPTION 'update policy publication revision must advance';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_policy_publication_state_advance
BEFORE UPDATE OF current_revision ON update_policy_publication_state
FOR EACH ROW EXECUTE FUNCTION guard_update_policy_publication_advance();

CREATE FUNCTION guard_update_policy_publication_immutability() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'published update policies are immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_policy_publications_immutable
BEFORE UPDATE OR DELETE ON update_policy_publications
FOR EACH ROW EXECUTE FUNCTION guard_update_policy_publication_immutability();
