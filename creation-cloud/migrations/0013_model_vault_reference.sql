-- 用生成列把“未墓碑”变成可声明的外键目标，由 PostgreSQL 处理并发引用完整性。
ALTER TABLE vault_envelopes
    ADD COLUMN active_reference_id UUID
    GENERATED ALWAYS AS (
        CASE WHEN deleted_at IS NULL THEN id ELSE NULL END
    ) STORED;

ALTER TABLE vault_envelopes
    ADD CONSTRAINT vault_envelopes_account_active_reference_key
    UNIQUE (account_id, active_reference_id);

ALTER TABLE model_profiles
    ADD CONSTRAINT model_profiles_active_vault_envelope_fkey
    FOREIGN KEY (account_id, vault_envelope_id)
    REFERENCES vault_envelopes(account_id, active_reference_id)
    ON UPDATE RESTRICT
    ON DELETE CASCADE;
