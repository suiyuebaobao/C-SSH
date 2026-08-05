-- AI provider API Key/Token is client-local only. Remove both retired personal-model stores.
DELETE FROM vault_envelopes
WHERE id IN (
    SELECT vault_envelope_id
    FROM model_profiles
    WHERE vault_envelope_id IS NOT NULL
);

DROP TABLE IF EXISTS model_profiles;
DROP TABLE IF EXISTS account_model_secrets;
