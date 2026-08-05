ALTER TABLE auth_settings
    ADD COLUMN email_cooldown_seconds INTEGER NOT NULL DEFAULT 60
        CHECK (email_cooldown_seconds BETWEEN 30 AND 3600),
    ADD COLUMN login_failure_threshold INTEGER NOT NULL DEFAULT 5
        CHECK (login_failure_threshold BETWEEN 3 AND 20),
    ADD COLUMN login_lockout_minutes INTEGER NOT NULL DEFAULT 30
        CHECK (login_lockout_minutes BETWEEN 1 AND 1440);

ALTER TABLE accounts
    ADD COLUMN consecutive_login_failures INTEGER NOT NULL DEFAULT 0
        CHECK (consecutive_login_failures >= 0),
    ADD COLUMN login_locked_until TIMESTAMPTZ;

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
        )
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid auth settings revision';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER TABLE audit_events
    DROP CONSTRAINT audit_events_auth_settings_v3_semantic_contract;

ALTER TABLE audit_events
ADD CONSTRAINT audit_events_auth_settings_v4_semantic_contract CHECK (
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
        AND (details->>'revision')::BIGINT >= 2
        AND (details->>'registration_challenges_invalidated')::BIGINT >= 0
        AND (details->>'user_login_challenges_invalidated')::BIGINT >= 0
        AND (details->>'user_captcha_challenges_invalidated')::BIGINT >= 0
        AND (details->>'admin_login_challenges_invalidated')::BIGINT >= 0
        AND (details->>'admin_captcha_challenges_invalidated')::BIGINT >= 0
    )
) NOT VALID;
