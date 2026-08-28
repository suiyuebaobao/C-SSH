-- 账号通知只保存固定 code、匿名资源 UUID 和空参数对象，不承载业务正文或秘密。
CREATE TABLE account_notifications (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    kind TEXT NOT NULL CHECK (kind IN ('account_security', 'sync')),
    priority TEXT NOT NULL CHECK (priority IN ('normal', 'important', 'critical')),
    code TEXT NOT NULL CHECK (
        (kind = 'account_security' AND code IN (
            'security_review_required',
            'password_changed',
            'device_revoked',
            'session_revoked'
        ))
        OR
        (kind = 'sync' AND code IN (
            'sync_review_required',
            'sync_upload_completed',
            'sync_download_completed',
            'sync_reset_completed'
        ))
    ),
    resource_id UUID,
    parameters JSONB NOT NULL DEFAULT '{}'::jsonb
        CHECK (parameters = '{}'::jsonb),
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (expires_at IS NULL OR expires_at > published_at),
    UNIQUE (id, account_id, revision)
);

CREATE INDEX account_notifications_account_cursor_idx
ON account_notifications(account_id, published_at DESC, id DESC);

CREATE TABLE account_notification_receipts (
    notification_id UUID NOT NULL,
    account_id UUID NOT NULL,
    notification_revision BIGINT NOT NULL CHECK (notification_revision > 0),
    read_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (notification_id, account_id),
    FOREIGN KEY (notification_id, account_id, notification_revision)
        REFERENCES account_notifications(id, account_id, revision)
        ON DELETE CASCADE
);

CREATE INDEX account_notification_receipts_account_idx
ON account_notification_receipts(account_id, read_at DESC);

CREATE FUNCTION guard_account_notification_immutability() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'account notifications are immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER account_notifications_immutable
BEFORE UPDATE ON account_notifications
FOR EACH ROW EXECUTE FUNCTION guard_account_notification_immutability();
