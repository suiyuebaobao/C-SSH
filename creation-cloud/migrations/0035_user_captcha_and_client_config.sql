ALTER TABLE auth_settings
ADD COLUMN user_captcha_enabled BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE admin_login_captcha_challenges
RENAME TO auth_captcha_challenges;

ALTER INDEX admin_login_captcha_open_idx
RENAME TO auth_captcha_open_idx;

ALTER TABLE auth_captcha_challenges
ADD COLUMN purpose TEXT NOT NULL DEFAULT 'admin_login'
    CHECK (purpose IN ('register', 'login', 'admin_login'));

ALTER TABLE auth_captcha_challenges
ALTER COLUMN purpose DROP DEFAULT;

UPDATE auth_captcha_challenges
SET consumed_at = now()
WHERE consumed_at IS NULL;

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
        )
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid auth settings revision';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER TABLE audit_events
ADD CONSTRAINT audit_events_auth_settings_v2_semantic_contract CHECK (
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
            'revision',
            'registration_challenges_invalidated',
            'login_challenges_invalidated',
            'captcha_challenges_invalidated'
        ]
        AND details - ARRAY[
            'email_verification_enabled',
            'user_captcha_enabled',
            'revision',
            'registration_challenges_invalidated',
            'login_challenges_invalidated',
            'captcha_challenges_invalidated'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'email_verification_enabled') = 'boolean'
        AND jsonb_typeof(details->'user_captcha_enabled') = 'boolean'
        AND jsonb_typeof(details->'revision') = 'number'
        AND jsonb_typeof(details->'registration_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'login_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'captcha_challenges_invalidated') = 'number'
        AND (details->>'revision')::BIGINT >= 2
        AND (details->>'registration_challenges_invalidated')::BIGINT >= 0
        AND (details->>'login_challenges_invalidated')::BIGINT >= 0
        AND (details->>'captcha_challenges_invalidated')::BIGINT >= 0
    )
);
