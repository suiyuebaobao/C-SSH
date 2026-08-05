CREATE TABLE auth_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    email_verification_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision >= 1),
    updated_by UUID REFERENCES accounts(id) ON DELETE SET NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO auth_settings (
    singleton,
    email_verification_enabled,
    revision,
    updated_by
) VALUES (TRUE, TRUE, 1, NULL);

CREATE FUNCTION guard_auth_settings_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'auth settings singleton cannot be deleted';
    END IF;

    IF NEW.singleton IS DISTINCT FROM OLD.singleton
        OR NEW.revision <> OLD.revision + 1
        OR NEW.email_verification_enabled IS NOT DISTINCT FROM OLD.email_verification_enabled
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid auth settings revision';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER auth_settings_guard
BEFORE UPDATE OR DELETE ON auth_settings
FOR EACH ROW EXECUTE FUNCTION guard_auth_settings_mutation();

ALTER TABLE audit_events
ADD CONSTRAINT audit_events_auth_settings_semantic_contract CHECK (
    action <> 'auth_settings.email_verification_updated'
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'auth_settings'
        AND resource_id = 'global'
        AND outcome = 'success'
        AND request_id IS NOT NULL
        AND request_id ~ '^[A-Za-z0-9._:-]{1,128}$'
        AND details ?& ARRAY[
            'email_verification_enabled',
            'revision',
            'registration_challenges_invalidated',
            'login_challenges_invalidated'
        ]
        AND details - ARRAY[
            'email_verification_enabled',
            'revision',
            'registration_challenges_invalidated',
            'login_challenges_invalidated'
        ] = '{}'::jsonb
        AND jsonb_typeof(details->'email_verification_enabled') = 'boolean'
        AND jsonb_typeof(details->'revision') = 'number'
        AND jsonb_typeof(details->'registration_challenges_invalidated') = 'number'
        AND jsonb_typeof(details->'login_challenges_invalidated') = 'number'
        AND (details->>'revision')::BIGINT >= 2
        AND (details->>'registration_challenges_invalidated')::BIGINT >= 0
        AND (details->>'login_challenges_invalidated')::BIGINT >= 0
    )
);
