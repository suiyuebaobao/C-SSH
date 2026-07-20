CREATE VIEW admin_user_overview AS
SELECT
    count(*)::BIGINT AS total_users,
    count(*) FILTER (WHERE status = 'active')::BIGINT AS active_users,
    count(*) FILTER (WHERE status = 'disabled')::BIGINT AS disabled_users,
    count(*) FILTER (WHERE role = 'admin')::BIGINT AS admin_users
FROM accounts;

CREATE VIEW admin_device_overview AS
SELECT
    count(*)::BIGINT AS total_devices,
    count(*) FILTER (WHERE revoked_at IS NULL)::BIGINT AS active_devices,
    count(*) FILTER (WHERE revoked_at IS NOT NULL)::BIGINT AS revoked_devices
FROM devices;

CREATE VIEW admin_release_overview AS
SELECT
    count(*)::BIGINT AS total_releases,
    count(*) FILTER (WHERE status = 'draft')::BIGINT AS draft_releases,
    count(*) FILTER (WHERE status = 'validating')::BIGINT AS validating_releases,
    count(*) FILTER (WHERE status = 'published')::BIGINT AS published_releases,
    count(*) FILTER (WHERE status = 'revoked')::BIGINT AS revoked_releases,
    count(*) FILTER (WHERE status = 'hidden')::BIGINT AS hidden_releases
FROM releases;

CREATE VIEW admin_audit_overview AS
SELECT
    count(*)::BIGINT AS total_events,
    count(*) FILTER (WHERE outcome = 'success')::BIGINT AS successful_events,
    count(*) FILTER (WHERE outcome = 'failure')::BIGINT AS failed_events,
    max(created_at) AS latest_event_at
FROM audit_events;
