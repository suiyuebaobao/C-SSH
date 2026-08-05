-- Device-specific host download scopes were removed. Every active device may
-- pull all hosts owned by its authenticated account.
DROP TABLE IF EXISTS cloud_host_download_allowlist;
