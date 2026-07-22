CREATE TABLE release_source_inspections (
    source_id UUID PRIMARY KEY REFERENCES release_sources(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (
        status IN ('healthy', 'missing', 'size_mismatch', 'hash_mismatch', 'io_error')
    ),
    inspected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    observed_byte_size BIGINT CHECK (observed_byte_size IS NULL OR observed_byte_size >= 0),
    observed_sha256 TEXT CHECK (
        observed_sha256 IS NULL OR observed_sha256 ~ '^[0-9a-f]{64}$'
    )
);

CREATE INDEX release_source_inspections_status_idx
    ON release_source_inspections(status, inspected_at DESC);

CREATE TABLE site_media_inspections (
    media_id UUID PRIMARY KEY REFERENCES site_media(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (
        status IN ('healthy', 'missing', 'size_mismatch', 'hash_mismatch', 'io_error')
    ),
    inspected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    observed_byte_size BIGINT CHECK (observed_byte_size IS NULL OR observed_byte_size >= 0),
    observed_sha256 TEXT CHECK (
        observed_sha256 IS NULL OR observed_sha256 ~ '^[0-9a-f]{64}$'
    )
);

CREATE INDEX site_media_inspections_status_idx
    ON site_media_inspections(status, inspected_at DESC);
