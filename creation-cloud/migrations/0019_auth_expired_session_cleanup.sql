CREATE INDEX sessions_expired_cleanup_idx
    ON sessions(expires_at, id);
