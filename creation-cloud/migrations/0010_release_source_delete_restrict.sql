ALTER TABLE release_sources
DROP CONSTRAINT release_sources_asset_id_fkey;

ALTER TABLE release_sources
ADD CONSTRAINT release_sources_asset_id_fkey
FOREIGN KEY (asset_id) REFERENCES release_assets(id) ON DELETE RESTRICT;

CREATE FUNCTION guard_release_asset_parent_identity() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.release_id IS DISTINCT FROM OLD.release_id THEN
        RAISE EXCEPTION 'asset release identity is immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER release_assets_parent_identity_guard
BEFORE UPDATE OF release_id ON release_assets
FOR EACH ROW EXECUTE FUNCTION guard_release_asset_parent_identity();

CREATE FUNCTION guard_release_source_parent_identity() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.asset_id IS DISTINCT FROM OLD.asset_id THEN
        RAISE EXCEPTION 'source asset identity is immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER release_sources_parent_identity_guard
BEFORE UPDATE OF asset_id ON release_sources
FOR EACH ROW EXECUTE FUNCTION guard_release_source_parent_identity();
