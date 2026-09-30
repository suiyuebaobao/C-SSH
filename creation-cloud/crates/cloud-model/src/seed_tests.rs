//! 系统模型补种清单与“不覆盖”语义测试。

use std::collections::HashSet;

use url::Url;

use crate::ReasoningControl;
use crate::repository::{
    ACTIVE_ADMIN_SQL, INSERT_SEED_SQL, LOCK_CATALOG_SQL, RETIRE_UNEDITED_SEED_SQL,
    RETIRED_SYSTEM_MODEL_IDS, SYSTEM_MODEL_SEEDS, UPDATE_UNEDITED_SEED_SQL,
};

#[test]
fn system_catalog_contains_only_the_sixteen_verified_models() {
    let names = SYSTEM_MODEL_SEEDS
        .iter()
        .map(|seed| seed.name)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "deepseek-v4-pro",
            "deepseek-v4-flash",
            "kimi-k3",
            "glm-5.2",
            "qwen3.7-max",
            "qwen3.7-plus",
            "qwen3.7-flash",
            "MiniMax-M3",
            "step-3.7-flash",
            "mimo-v2.5-pro",
            "mimo-v2.5",
            "ernie-5.1",
            "hy3",
            "spark-x",
            "Baichuan4-Turbo",
            "yi-large",
        ]
    );
    assert_eq!(
        names.iter().filter(|name| name.starts_with("kimi")).count(),
        1
    );
    assert_eq!(RETIRED_SYSTEM_MODEL_IDS.len(), 2);
}

#[test]
fn system_catalog_has_unique_fixed_ids_ascii_providers_and_http_urls() {
    let ids = SYSTEM_MODEL_SEEDS
        .iter()
        .map(|seed| seed.id)
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), SYSTEM_MODEL_SEEDS.len());
    for seed in SYSTEM_MODEL_SEEDS {
        assert!(!seed.id.is_nil());
        assert!(
            seed.provider.chars().all(|character| {
                character.is_ascii_alphanumeric() || ".-_".contains(character)
            }),
            "provider 必须符合现有 ASCII slug 门禁：{}",
            seed.provider
        );
        assert!((4_096..=2_000_000).contains(&seed.context_length));
        assert_eq!(
            seed.openai_base_url.is_some(),
            seed.openai_model_name.is_some()
        );
        assert_eq!(
            seed.anthropic_base_url.is_some(),
            seed.anthropic_model_name.is_some()
        );
        assert!(seed.openai_base_url.is_some() || seed.anthropic_base_url.is_some());
        assert_eq!(seed.reasoning_control, ReasoningControl::Unsupported);
        for raw_url in [seed.openai_base_url, seed.anthropic_base_url]
            .into_iter()
            .flatten()
        {
            let url = Url::parse(raw_url).expect("预置 API URL 必须可解析");
            assert_eq!(url.scheme(), "https");
            assert!(url.host_str().is_some());
            assert!(url.username().is_empty());
            assert!(url.password().is_none());
            assert!(url.query().is_none());
            assert!(url.fragment().is_none());
        }
    }
}

#[test]
fn seed_requires_an_active_admin_and_updates_only_unedited_system_rows() {
    assert!(ACTIVE_ADMIN_SQL.contains("role = 'admin' AND status = 'active'"));
    assert!(ACTIVE_ADMIN_SQL.contains("FOR SHARE"));
    assert!(LOCK_CATALOG_SQL.contains("SHARE ROW EXCLUSIVE"));
    assert!(INSERT_SEED_SQL.contains("openai_base_url"));
    assert!(INSERT_SEED_SQL.contains("anthropic_base_url"));
    assert!(INSERT_SEED_SQL.contains("reasoning_control"));
    assert!(INSERT_SEED_SQL.contains("TRUE, FALSE"));
    assert!(INSERT_SEED_SQL.contains("created_by, updated_by"));
    assert!(INSERT_SEED_SQL.contains("$11, $11"));
    assert!(INSERT_SEED_SQL.contains("system_seeded"));
    assert!(INSERT_SEED_SQL.contains("ON CONFLICT DO NOTHING"));
    assert!(!INSERT_SEED_SQL.contains("DO UPDATE"));
    assert!(UPDATE_UNEDITED_SEED_SQL.contains("system_seeded"));
    assert!(UPDATE_UNEDITED_SEED_SQL.contains("revision = 1"));
    assert!(UPDATE_UNEDITED_SEED_SQL.contains("deleted_at IS NULL"));
    assert!(UPDATE_UNEDITED_SEED_SQL.contains("revision = revision + 1"));
    assert!(UPDATE_UNEDITED_SEED_SQL.contains("reasoning_control = $8"));
    assert!(RETIRE_UNEDITED_SEED_SQL.contains("enabled = FALSE"));
    assert!(RETIRE_UNEDITED_SEED_SQL.contains("revision = 1"));
}

#[test]
fn official_contexts_and_distinct_kimi_model_ids_are_preserved() {
    let kimi = SYSTEM_MODEL_SEEDS
        .iter()
        .find(|seed| seed.name == "kimi-k3")
        .expect("Kimi K3 预置应存在");
    assert_eq!(kimi.context_length, 1_048_576);
    assert_eq!(kimi.openai_model_name, Some("kimi-k3"));
    assert_eq!(kimi.anthropic_model_name, Some("kimi-k3[1m]"));
    assert_eq!(kimi.reasoning_control, ReasoningControl::Unsupported);

    let contexts = SYSTEM_MODEL_SEEDS
        .iter()
        .map(|seed| (seed.name, seed.context_length))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(contexts["spark-x"], 192_000);
    assert_eq!(contexts["Baichuan4-Turbo"], 32_000);
    assert_eq!(contexts["yi-large"], 32_000);
}
