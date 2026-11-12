-- 设备名额与安装同步身份分离；原登记、密文、游标及审计原样保留。
ALTER TABLE devices ADD COLUMN physical_device_fingerprint TEXT
    CHECK (physical_device_fingerprint IS NULL OR physical_device_fingerprint ~ '^[0-9a-f]{64}$');
CREATE INDEX devices_active_physical_identity_idx
    ON devices(account_id, platform, physical_device_fingerprint)
    WHERE revoked_at IS NULL AND physical_device_fingerprint IS NOT NULL;

CREATE FUNCTION guard_device_physical_identity() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.physical_device_fingerprint IS NOT NULL
        AND NEW.physical_device_fingerprint IS DISTINCT FROM OLD.physical_device_fingerprint THEN
        RAISE EXCEPTION 'bound device identity cannot be changed';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER device_physical_identity_immutable
BEFORE UPDATE OF physical_device_fingerprint ON devices
FOR EACH ROW EXECUTE FUNCTION guard_device_physical_identity();

ALTER TABLE audit_events ADD CONSTRAINT audit_events_device_identity_semantic_contract CHECK (
    action <> 'device.identity_bound' OR (
        actor_account_id IS NOT NULL AND resource_kind='device'
        AND resource_id ~ '^[0-9a-f-]{36}$' AND outcome='success'
        AND request_id IS NOT NULL AND request_id ~ '^[A-Za-z0-9._:-]{1,128}$'
        AND details = '{"identity_version":1}'::JSONB
    )
) NOT VALID;

CREATE VIEW device_identity_groups AS
SELECT account_id, platform,
       COALESCE(physical_device_fingerprint, 'legacy:' || id::TEXT) AS identity_key,
       BOOL_OR(revoked_at IS NULL) AS is_active,
       (ARRAY_AGG(id ORDER BY (revoked_at IS NULL) DESC,
           last_seen_at DESC NULLS LAST, created_at DESC, id))[1] AS representative_id,
       ARRAY_AGG(id ORDER BY id) AS registration_ids,
       COUNT(*)::BIGINT AS registration_count
FROM devices
GROUP BY account_id, platform, COALESCE(physical_device_fingerprint, 'legacy:' || id::TEXT);

CREATE OR REPLACE VIEW admin_device_overview AS
SELECT count(*)::BIGINT AS total_devices,
       count(*) FILTER (WHERE is_active)::BIGINT AS active_devices,
       count(*) FILTER (WHERE NOT is_active)::BIGINT AS revoked_devices
FROM device_identity_groups;

ALTER TABLE auth_settings ADD COLUMN max_active_devices_per_account INTEGER NOT NULL DEFAULT 16
    CHECK (max_active_devices_per_account BETWEEN 1 AND 1000);

CREATE TABLE account_device_policies (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    maximum INTEGER CHECK (maximum BETWEEN 1 AND 1000),
    revision BIGINT NOT NULL CHECK (revision > 0),
    updated_by UUID NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
COMMENT ON COLUMN account_device_policies.updated_by IS
    'Actor identity at update time; account lifecycle does not rewrite historical identity.';

CREATE FUNCTION guard_account_device_policy() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.account_id IS DISTINCT FROM OLD.account_id
        OR NEW.revision <> OLD.revision + 1
        OR NEW.maximum IS NOT DISTINCT FROM OLD.maximum
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid account device policy revision';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
CREATE TRIGGER account_device_policy_revision
BEFORE UPDATE ON account_device_policies
FOR EACH ROW EXECUTE FUNCTION guard_account_device_policy();

ALTER TABLE audit_events ADD CONSTRAINT audit_events_device_policy_semantic_contract CHECK (
    action <> 'device_policy.updated' OR (
        actor_account_id IS NOT NULL AND resource_kind = 'account_device_policy'
        AND resource_id ~ '^[0-9a-f-]{36}$' AND outcome = 'success'
        AND request_id IS NOT NULL AND request_id ~ '^[A-Za-z0-9._:-]{1,128}$'
        AND details ?& ARRAY['maximum','effective_maximum','revision']
        AND details - ARRAY['maximum','effective_maximum','revision'] = '{}'::JSONB
        AND (jsonb_typeof(details->'maximum') = 'null' OR (
            jsonb_typeof(details->'maximum') = 'number'
            AND (details->>'maximum')::INTEGER BETWEEN 1 AND 1000))
        AND jsonb_typeof(details->'effective_maximum') = 'number'
        AND (details->>'effective_maximum')::INTEGER BETWEEN 1 AND 1000
        AND jsonb_typeof(details->'revision') = 'number'
        AND (details->>'revision')::BIGINT > 0
    )
) NOT VALID;

CREATE OR REPLACE FUNCTION guard_auth_settings_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'auth settings singleton cannot be deleted';
    END IF;

    IF NEW.singleton IS DISTINCT FROM OLD.singleton
        OR NEW.revision <> OLD.revision + 1
        OR (
            NEW.email_verification_enabled IS NOT DISTINCT FROM OLD.email_verification_enabled
            AND NEW.user_captcha_enabled IS NOT DISTINCT FROM OLD.user_captcha_enabled
            AND NEW.admin_email_verification_enabled IS NOT DISTINCT FROM OLD.admin_email_verification_enabled
            AND NEW.admin_captcha_enabled IS NOT DISTINCT FROM OLD.admin_captcha_enabled
            AND NEW.email_cooldown_seconds IS NOT DISTINCT FROM OLD.email_cooldown_seconds
            AND NEW.login_failure_threshold IS NOT DISTINCT FROM OLD.login_failure_threshold
            AND NEW.login_lockout_minutes IS NOT DISTINCT FROM OLD.login_lockout_minutes
            AND NEW.max_active_devices_per_account IS NOT DISTINCT FROM OLD.max_active_devices_per_account
        )
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid auth settings revision';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER TABLE audit_events DROP CONSTRAINT audit_events_auth_settings_v4_semantic_contract;
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_auth_settings_v5_semantic_contract CHECK (
    action <> 'auth_settings.updated'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'auth_settings'
        AND resource_id = 'global'
        AND outcome = 'success'
        AND request_id IS NOT NULL
        AND request_id ~ '^[A-Za-z0-9._:-]{1,128}$'
        AND details ?& ARRAY[
            'email_verification_enabled',
            'user_captcha_enabled',
            'admin_email_verification_enabled',
            'admin_captcha_enabled',
            'email_cooldown_seconds',
            'login_failure_threshold',
            'login_lockout_minutes',
            'revision',
            'registration_challenges_invalidated',
            'user_login_challenges_invalidated',
            'user_captcha_challenges_invalidated',
            'admin_login_challenges_invalidated',
            'admin_captcha_challenges_invalidated'
        ]
        AND details - ARRAY[
            'email_verification_enabled',
            'user_captcha_enabled',
            'admin_email_verification_enabled',
            'admin_captcha_enabled',
            'email_cooldown_seconds',
            'login_failure_threshold',
            'login_lockout_minutes',
            'max_active_devices_per_account',
            'revision',
            'registration_challenges_invalidated',
            'user_login_challenges_invalidated',
            'user_captcha_challenges_invalidated',
            'admin_login_challenges_invalidated',
            'admin_captcha_challenges_invalidated'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'email_verification_enabled') = 'boolean'
        AND jsonb_typeof(details->'user_captcha_enabled') = 'boolean'
        AND jsonb_typeof(details->'admin_email_verification_enabled') = 'boolean'
        AND jsonb_typeof(details->'admin_captcha_enabled') = 'boolean'
        AND jsonb_typeof(details->'email_cooldown_seconds') = 'number'
        AND jsonb_typeof(details->'login_failure_threshold') = 'number'
        AND jsonb_typeof(details->'login_lockout_minutes') = 'number'
        AND jsonb_typeof(details->'revision') = 'number'
        AND jsonb_typeof(details->'registration_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'user_login_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'user_captcha_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'admin_login_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'admin_captcha_challenges_invalidated') = 'number'
        AND (details->>'email_cooldown_seconds')::INTEGER BETWEEN 30 AND 3600
        AND (details->>'login_failure_threshold')::INTEGER BETWEEN 3 AND 20
        AND (details->>'login_lockout_minutes')::INTEGER BETWEEN 1 AND 1440
        AND (NOT details ? 'max_active_devices_per_account' OR (
            jsonb_typeof(details->'max_active_devices_per_account') = 'number'
            AND (details->>'max_active_devices_per_account')::INTEGER BETWEEN 1 AND 1000
        ))
        AND (details->>'revision')::BIGINT >= 2
        AND (details->>'registration_challenges_invalidated')::BIGINT >= 0
        AND (details->>'user_login_challenges_invalidated')::BIGINT >= 0
        AND (details->>'user_captcha_challenges_invalidated')::BIGINT >= 0
        AND (details->>'admin_login_challenges_invalidated')::BIGINT >= 0
        AND (details->>'admin_captcha_challenges_invalidated')::BIGINT >= 0
    )
) NOT VALID;
