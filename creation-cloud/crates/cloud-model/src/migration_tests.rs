//! 模型目录接口格式迁移的静态边界测试。

const MIGRATION: &str = include_str!("../../../migrations/0031_model_catalog_api_format.sql");
const INTERFACES_MIGRATION: &str =
    include_str!("../../../migrations/0043_model_catalog_interfaces.sql");
const RESPONSES_REASONING_MIGRATION: &str =
    include_str!("../../../migrations/0047_model_catalog_responses_reasoning.sql");
const REASONING_PROFILES_MIGRATION: &str =
    include_str!("../../../migrations/0048_model_catalog_reasoning_profiles.sql");
const REMOVE_ACCOUNT_MODEL_SECRETS: &str =
    include_str!("../../../migrations/0040_remove_account_model_secrets.sql");

#[test]
fn migration_adds_bounded_api_format_and_system_seed_marker() {
    assert!(MIGRATION.contains("ADD COLUMN api_format TEXT NOT NULL"));
    assert!(MIGRATION.contains("'openai_compatible'"));
    assert!(MIGRATION.contains("'anthropic_compatible'"));
    assert!(MIGRATION.contains("global_model_catalog_api_format_check"));
    assert!(MIGRATION.contains("ADD COLUMN system_seeded BOOLEAN NOT NULL DEFAULT FALSE"));
}

#[test]
fn migration_does_not_relax_real_admin_foreign_keys() {
    assert!(!MIGRATION.contains("DROP COLUMN created_by"));
    assert!(!MIGRATION.contains("DROP COLUMN updated_by"));
    assert!(!MIGRATION.contains("DROP CONSTRAINT"));
}

#[test]
fn interfaces_migration_backfills_then_removes_the_singular_contract() {
    for column in [
        "openai_base_url",
        "openai_model_name",
        "anthropic_base_url",
        "anthropic_model_name",
    ] {
        assert!(INTERFACES_MIGRATION.contains(&format!("ADD COLUMN {column} TEXT")));
    }
    assert!(INTERFACES_MIGRATION.contains("WHEN api_format = 'openai_compatible'"));
    assert!(INTERFACES_MIGRATION.contains("WHEN api_format = 'anthropic_compatible'"));
    assert!(INTERFACES_MIGRATION.contains("DROP COLUMN api_format"));
    assert!(INTERFACES_MIGRATION.contains("DROP COLUMN base_url"));
    assert!(INTERFACES_MIGRATION.contains("DROP COLUMN model_name"));
}

#[test]
fn interfaces_migration_fails_closed_without_fabricating_an_endpoint() {
    assert!(INTERFACES_MIGRATION.contains("WHERE base_url IS NULL"));
    assert!(INTERFACES_MIGRATION.contains("RAISE EXCEPTION"));
    assert!(!INTERFACES_MIGRATION.contains("unconfigured.invalid"));
    assert!(INTERFACES_MIGRATION.contains("global_model_catalog_openai_pair_check"));
    assert!(INTERFACES_MIGRATION.contains("global_model_catalog_anthropic_pair_check"));
    assert!(INTERFACES_MIGRATION.contains("global_model_catalog_interface_required_check"));
    assert!(INTERFACES_MIGRATION.contains("context_length BETWEEN 4096 AND 2000000"));
}

#[test]
fn forward_migration_adds_responses_and_typed_reasoning_without_guessing() {
    for column in ["responses_base_url", "responses_model_name"] {
        assert!(RESPONSES_REASONING_MIGRATION.contains(&format!("ADD COLUMN {column} TEXT")));
    }
    assert!(
        RESPONSES_REASONING_MIGRATION
            .contains("ADD COLUMN reasoning_control TEXT NOT NULL DEFAULT 'unsupported'")
    );
    assert!(
        RESPONSES_REASONING_MIGRATION.contains("reasoning_control IN ('unsupported', 'deepseek')")
    );
    assert!(RESPONSES_REASONING_MIGRATION.contains("responses_base_url IS NOT NULL"));
    assert!(!RESPONSES_REASONING_MIGRATION.contains("capability_tags @>"));
    assert!(!RESPONSES_REASONING_MIGRATION.contains("model_name LIKE"));
}

#[test]
fn append_only_reasoning_migration_normalizes_then_rejects_vendor_profiles() {
    assert!(
        REASONING_PROFILES_MIGRATION
            .contains("DROP CONSTRAINT global_model_catalog_reasoning_control_check")
    );
    assert!(REASONING_PROFILES_MIGRATION.contains("UPDATE global_model_catalog"));
    assert!(REASONING_PROFILES_MIGRATION.contains("SET reasoning_control = 'unsupported'"));
    assert!(
        REASONING_PROFILES_MIGRATION
            .contains("WHERE reasoning_control IS DISTINCT FROM 'unsupported'")
    );
    assert!(REASONING_PROFILES_MIGRATION.contains("revision = revision + 1"));
    assert!(REASONING_PROFILES_MIGRATION.contains("updated_at + interval '1 microsecond'"));
    assert!(REASONING_PROFILES_MIGRATION.contains("reasoning_control = 'unsupported'"));
    let drop_position = REASONING_PROFILES_MIGRATION
        .find("DROP CONSTRAINT global_model_catalog_reasoning_control_check")
        .expect("应先移除 0047 两值约束");
    let update_position = REASONING_PROFILES_MIGRATION
        .find("UPDATE global_model_catalog")
        .expect("应归一既有值");
    let add_position = REASONING_PROFILES_MIGRATION
        .find("ADD CONSTRAINT global_model_catalog_reasoning_control_check")
        .expect("应建立单值约束");
    assert!(drop_position < update_position && update_position < add_position);
    for forbidden in [
        "'openai'",
        "'deepseek'",
        "'glm'",
        "'qwen'",
        "'kimi'",
        "'minimax'",
    ] {
        assert!(!REASONING_PROFILES_MIGRATION.contains(forbidden));
    }
    assert!(!REASONING_PROFILES_MIGRATION.contains("cc310000-0000-4000-8000"));
    assert!(!REASONING_PROFILES_MIGRATION.contains("FROM ("));
    assert!(!REASONING_PROFILES_MIGRATION.contains("provider ="));
    assert!(!REASONING_PROFILES_MIGRATION.contains("model_name LIKE"));
    assert!(!REASONING_PROFILES_MIGRATION.contains("base_url LIKE"));
    assert!(!REASONING_PROFILES_MIGRATION.contains("ADD COLUMN reasoning_control"));
}

#[test]
fn forward_migration_removes_only_the_retired_model_secret_store() {
    assert!(REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE IF EXISTS account_model_secrets"));
    assert!(REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE IF EXISTS model_profiles"));
    assert!(REMOVE_ACCOUNT_MODEL_SECRETS.contains("DELETE FROM vault_envelopes"));
    assert!(REMOVE_ACCOUNT_MODEL_SECRETS.contains("WHERE vault_envelope_id IS NOT NULL"));
    assert!(!REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE global_model_catalog"));
    assert!(!REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE IF EXISTS global_model_catalog"));
    assert!(!REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE vault_envelopes"));
    assert!(!REMOVE_ACCOUNT_MODEL_SECRETS.contains("DROP TABLE IF EXISTS vault_envelopes"));
}
