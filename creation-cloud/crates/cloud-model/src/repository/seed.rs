//! 中国厂商默认模型目录的幂等补种。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

use crate::ReasoningControl;

use super::storage;

pub(crate) const ACTIVE_ADMIN_SQL: &str = r#"
    SELECT id
    FROM accounts
    WHERE role = 'admin' AND status = 'active'
    ORDER BY created_at ASC, id ASC
    LIMIT 1
    FOR SHARE
"#;

pub(crate) const LOCK_CATALOG_SQL: &str =
    "LOCK TABLE global_model_catalog IN SHARE ROW EXCLUSIVE MODE";

pub(crate) const INSERT_SEED_SQL: &str = r#"
    INSERT INTO global_model_catalog (
        id, name, provider, openai_base_url, openai_model_name,
        anthropic_base_url, anthropic_model_name, reasoning_control,
        context_length, capability_tags, default_parameters,
        enabled, is_default, sort_order, created_by, updated_by,
        system_seeded
    )
    VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8,
        $9, '{}'::text[], '{}'::jsonb,
        TRUE, FALSE, $10, $11, $11, TRUE
    )
    ON CONFLICT DO NOTHING
"#;

pub(crate) const UPDATE_UNEDITED_SEED_SQL: &str = r#"
    UPDATE global_model_catalog
    SET name = $2,
        provider = $3,
        openai_base_url = $4,
        openai_model_name = $5,
        anthropic_base_url = $6,
        anthropic_model_name = $7,
        reasoning_control = $8,
        context_length = $9,
        enabled = TRUE,
        is_default = FALSE,
        sort_order = $10,
        revision = revision + 1,
        updated_by = $11,
        updated_at = now()
    WHERE id = $1
      AND system_seeded
      AND revision = 1
      AND deleted_at IS NULL
      AND ROW(
          name, provider, openai_base_url, openai_model_name,
          anthropic_base_url, anthropic_model_name, reasoning_control, context_length,
          enabled, is_default, sort_order
      ) IS DISTINCT FROM ROW(
          $2, $3, $4, $5, $6, $7, $8, $9,
          TRUE, FALSE, $10
      )
"#;

pub(crate) const RETIRE_UNEDITED_SEED_SQL: &str = r#"
    UPDATE global_model_catalog
    SET enabled = FALSE,
        is_default = FALSE,
        revision = revision + 1,
        updated_by = $2,
        updated_at = now()
    WHERE id = $1
      AND system_seeded
      AND revision = 1
      AND deleted_at IS NULL
      AND (enabled OR is_default)
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SystemModelSeed {
    pub(crate) id: Uuid,
    pub(crate) name: &'static str,
    pub(crate) provider: &'static str,
    pub(crate) context_length: i32,
    pub(crate) openai_base_url: Option<&'static str>,
    pub(crate) openai_model_name: Option<&'static str>,
    pub(crate) anthropic_base_url: Option<&'static str>,
    pub(crate) anthropic_model_name: Option<&'static str>,
    pub(crate) reasoning_control: ReasoningControl,
}

pub(crate) const SYSTEM_MODEL_SEEDS: [SystemModelSeed; 16] = [
    seed(
        1,
        "deepseek-v4-pro",
        "DeepSeek",
        1_000_000,
        Some(("https://api.deepseek.com", "deepseek-v4-pro")),
        Some(("https://api.deepseek.com/anthropic", "deepseek-v4-pro")),
        ReasoningControl::Unsupported,
    ),
    seed(
        2,
        "deepseek-v4-flash",
        "DeepSeek",
        1_000_000,
        Some(("https://api.deepseek.com", "deepseek-v4-flash")),
        Some(("https://api.deepseek.com/anthropic", "deepseek-v4-flash")),
        ReasoningControl::Unsupported,
    ),
    seed(
        3,
        "kimi-k3",
        "Kimi",
        1_048_576,
        Some(("https://api.moonshot.cn/v1", "kimi-k3")),
        Some(("https://api.moonshot.cn/anthropic", "kimi-k3[1m]")),
        ReasoningControl::Unsupported,
    ),
    seed(
        4,
        "glm-5.2",
        "ZhipuAI",
        1_000_000,
        Some(("https://open.bigmodel.cn/api/paas/v4", "glm-5.2")),
        None,
        ReasoningControl::Unsupported,
    ),
    seed(
        5,
        "qwen3.7-max",
        "Qwen",
        1_000_000,
        Some((
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen3.7-max",
        )),
        Some((
            "https://dashscope.aliyuncs.com/apps/anthropic",
            "qwen3.7-max",
        )),
        ReasoningControl::Unsupported,
    ),
    seed(
        6,
        "qwen3.7-plus",
        "Qwen",
        1_000_000,
        Some((
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen3.7-plus",
        )),
        Some((
            "https://dashscope.aliyuncs.com/apps/anthropic",
            "qwen3.7-plus",
        )),
        ReasoningControl::Unsupported,
    ),
    seed(
        7,
        "qwen3.7-flash",
        "Qwen",
        1_000_000,
        Some((
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "qwen3.7-flash",
        )),
        Some((
            "https://dashscope.aliyuncs.com/apps/anthropic",
            "qwen3.7-flash",
        )),
        ReasoningControl::Unsupported,
    ),
    seed(
        8,
        "MiniMax-M3",
        "MiniMax",
        1_000_000,
        Some(("https://api.minimaxi.com/v1", "MiniMax-M3")),
        Some(("https://api.minimaxi.com/anthropic", "MiniMax-M3")),
        ReasoningControl::Unsupported,
    ),
    seed(
        9,
        "step-3.7-flash",
        "StepFun",
        256_000,
        Some(("https://api.stepfun.com/v1", "step-3.7-flash")),
        Some(("https://api.stepfun.com", "step-3.7-flash")),
        ReasoningControl::Unsupported,
    ),
    seed(
        10,
        "mimo-v2.5-pro",
        "XiaomiMiMo",
        1_000_000,
        Some(("https://api.xiaomimimo.com/v1", "mimo-v2.5-pro")),
        Some(("https://api.xiaomimimo.com/anthropic", "mimo-v2.5-pro")),
        ReasoningControl::Unsupported,
    ),
    seed(
        11,
        "mimo-v2.5",
        "XiaomiMiMo",
        1_000_000,
        Some(("https://api.xiaomimimo.com/v1", "mimo-v2.5")),
        Some(("https://api.xiaomimimo.com/anthropic", "mimo-v2.5")),
        ReasoningControl::Unsupported,
    ),
    seed(
        13,
        "ernie-5.1",
        "Baidu",
        128_000,
        Some(("https://qianfan.baidubce.com/v2", "ernie-5.1")),
        Some(("https://qianfan.baidubce.com/anthropic", "ernie-5.1")),
        ReasoningControl::Unsupported,
    ),
    seed(
        14,
        "hy3",
        "Tencent",
        256_000,
        Some(("https://tokenhub.tencentmaas.com/v1", "hy3")),
        Some(("https://tokenhub.tencentmaas.com", "hy3")),
        ReasoningControl::Unsupported,
    ),
    seed(
        15,
        "spark-x",
        "iFlytek",
        192_000,
        Some(("https://spark-api-open.xf-yun.com/x2", "spark-x")),
        None,
        ReasoningControl::Unsupported,
    ),
    seed(
        17,
        "Baichuan4-Turbo",
        "Baichuan",
        32_000,
        Some(("https://api.baichuan-ai.com/v1", "Baichuan4-Turbo")),
        None,
        ReasoningControl::Unsupported,
    ),
    seed(
        18,
        "yi-large",
        "01AI",
        32_000,
        Some(("https://api.01.ai/v1", "yi-large")),
        None,
        ReasoningControl::Unsupported,
    ),
];

pub(crate) const RETIRED_SYSTEM_MODEL_IDS: [Uuid; 2] = [seed_id(12), seed_id(16)];

const fn seed(
    suffix: u128,
    name: &'static str,
    provider: &'static str,
    context_length: i32,
    openai: Option<(&'static str, &'static str)>,
    anthropic: Option<(&'static str, &'static str)>,
    reasoning_control: ReasoningControl,
) -> SystemModelSeed {
    SystemModelSeed {
        id: seed_id(suffix),
        name,
        provider,
        context_length,
        openai_base_url: interface_base_url(openai),
        openai_model_name: interface_model_name(openai),
        anthropic_base_url: interface_base_url(anthropic),
        anthropic_model_name: interface_model_name(anthropic),
        reasoning_control,
    }
}

const fn interface_base_url(
    interface: Option<(&'static str, &'static str)>,
) -> Option<&'static str> {
    match interface {
        Some((base_url, _)) => Some(base_url),
        None => None,
    }
}

const fn interface_model_name(
    interface: Option<(&'static str, &'static str)>,
) -> Option<&'static str> {
    match interface {
        Some((_, model_name)) => Some(model_name),
        None => None,
    }
}

const fn seed_id(suffix: u128) -> Uuid {
    Uuid::from_u128(0xcc31_0000_0000_4000_8000_0000_0000_0000 | suffix)
}

pub(crate) async fn seed_system_catalog(pool: &PgPool) -> AppResult<u64> {
    let mut transaction = pool
        .begin()
        .await
        .map_err(storage("无法开始默认模型补种事务"))?;
    let Some(actor_id) = sqlx::query_scalar::<_, Uuid>(ACTIVE_ADMIN_SQL)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage("无法读取有效管理员"))?
    else {
        transaction
            .commit()
            .await
            .map_err(storage("无法提交空的默认模型补种事务"))?;
        return Ok(0);
    };

    sqlx::query(LOCK_CATALOG_SQL)
        .execute(&mut *transaction)
        .await
        .map_err(storage("无法锁定默认模型目录"))?;

    let mut changed = 0_u64;
    for (index, seed) in SYSTEM_MODEL_SEEDS.iter().enumerate() {
        let sort_order = i32::try_from(index + 1)
            .map_err(|_| storage_overflow())?
            .saturating_mul(10);
        changed += sqlx::query(UPDATE_UNEDITED_SEED_SQL)
            .bind(seed.id)
            .bind(seed.name)
            .bind(seed.provider)
            .bind(seed.openai_base_url)
            .bind(seed.openai_model_name)
            .bind(seed.anthropic_base_url)
            .bind(seed.anthropic_model_name)
            .bind(seed.reasoning_control.as_str())
            .bind(seed.context_length)
            .bind(sort_order)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage("无法更新未编辑的默认模型"))?
            .rows_affected();
        changed += sqlx::query(INSERT_SEED_SQL)
            .bind(seed.id)
            .bind(seed.name)
            .bind(seed.provider)
            .bind(seed.openai_base_url)
            .bind(seed.openai_model_name)
            .bind(seed.anthropic_base_url)
            .bind(seed.anthropic_model_name)
            .bind(seed.reasoning_control.as_str())
            .bind(seed.context_length)
            .bind(sort_order)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage("无法补种默认模型"))?
            .rows_affected();
    }

    for id in RETIRED_SYSTEM_MODEL_IDS {
        changed += sqlx::query(RETIRE_UNEDITED_SEED_SQL)
            .bind(id)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage("无法停用未确认的默认模型"))?
            .rows_affected();
    }

    transaction
        .commit()
        .await
        .map_err(storage("无法提交默认模型补种事务"))?;
    Ok(changed)
}

fn storage_overflow() -> cloud_domain::AppError {
    cloud_domain::AppError::Internal("默认模型排序超出支持范围".to_owned())
}
