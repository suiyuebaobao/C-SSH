ALTER TABLE download_events
    ADD COLUMN aggregated_at TIMESTAMPTZ;

CREATE INDEX download_events_unaggregated_idx
    ON download_events(occurred_at, id)
    WHERE aggregated_at IS NULL;

CREATE TABLE download_event_daily_aggregates (
    bucket_date DATE NOT NULL,
    asset_id UUID NOT NULL REFERENCES release_assets(id) ON DELETE RESTRICT,
    source_id UUID NOT NULL REFERENCES release_sources(id) ON DELETE RESTRICT,
    audience TEXT NOT NULL CHECK (audience IN ('anonymous', 'authenticated')),
    event_count BIGINT NOT NULL CHECK (event_count > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (bucket_date, asset_id, source_id, audience),
    CHECK (updated_at >= created_at)
);

CREATE INDEX download_event_daily_aggregates_asset_idx
    ON download_event_daily_aggregates(asset_id, bucket_date DESC, source_id, audience);
