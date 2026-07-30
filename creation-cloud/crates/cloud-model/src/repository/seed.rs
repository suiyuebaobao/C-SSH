//! 中国厂商默认模型目录的幂等补种。

use cloud_domain::AppResult;
use cloud_store::PgPool;
use uuid::Uuid;

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
        id, name, provider, api_format, base_url, model_name,
        context_length, capability_tags, default_parameters,
        enabled, is_default, sort_order, created_by, updated_by,
        system_seeded
    )
    SELECT
        $1, $2, $3, 'openai_compatible', $4, $2,
        128000, '{}'::text[], '{}'::jsonb,
        TRUE, FALSE, $5, $6, $6,
        TRUE
    WHERE NOT EXISTS (
        SELECT 1
        FROM global_model_catalog
        WHERE provider = $3 AND model_name = $2
    )
    ON CONFLICT DO NOTHING
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SystemModelSeed {
    pub(crate) id: Uuid,
    pub(crate) model_name: &'static str,
    pub(crate) provider: &'static str,
    pub(crate) base_url: &'static str,
}

pub(crate) const SYSTEM_MODEL_SEEDS: [SystemModelSeed; 18] = [
    seed(1, "deepseek-v4-pro", "DeepSeek", "https://api.deepseek.com"),
    seed(
        2,
        "deepseek-v4-flash",
        "DeepSeek",
        "https://api.deepseek.com",
    ),
    seed(3, "kimi-k3", "Kimi", "https://api.moonshot.cn/v1"),
    seed(
        4,
        "glm-5.2",
        "ZhipuAI",
        "https://open.bigmodel.cn/api/paas/v4",
    ),
    seed(
        5,
        "qwen3.7-max",
        "Qwen",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
    ),
    seed(
        6,
        "qwen3.7-plus",
        "Qwen",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
    ),
    seed(
        7,
        "qwen3.7-flash",
        "Qwen",
        "https://dashscope.aliyuncs.com/compatible-mode/v1",
    ),
    seed(8, "MiniMax-M3", "MiniMax", "https://api.minimaxi.com/v1"),
    seed(9, "step-3.7-flash", "StepFun", "https://api.stepfun.com/v1"),
    seed(
        10,
        "mimo-v2.5-pro",
        "XiaomiMiMo",
        "https://api.xiaomimimo.com/v1",
    ),
    seed(
        11,
        "mimo-v2.5",
        "XiaomiMiMo",
        "https://api.xiaomimimo.com/v1",
    ),
    seed(
        12,
        "doubao-seed-2-0-pro-260215",
        "Doubao",
        "https://ark.cn-beijing.volces.com/api/v3",
    ),
    seed(13, "ernie-5.1", "Baidu", "https://qianfan.baidubce.com/v2"),
    seed(14, "hy3", "Tencent", "https://tokenhub.tencentmaas.com/v1"),
    seed(
        15,
        "spark-x",
        "iFlytek",
        "https://spark-api-open.xf-yun.com/x2/",
    ),
    seed(
        16,
        "SenseNova-V6-5-Pro",
        "SenseNova",
        "https://api.sensenova.cn/compatible-mode/v2",
    ),
    seed(
        17,
        "Baichuan4-Turbo",
        "Baichuan",
        "https://api.baichuan-ai.com/v1",
    ),
    seed(18, "yi-large", "01AI", "https://api.01.ai/v1"),
];

const fn seed(
    suffix: u128,
    model_name: &'static str,
    provider: &'static str,
    base_url: &'static str,
) -> SystemModelSeed {
    SystemModelSeed {
        id: Uuid::from_u128(0xcc31_0000_0000_4000_8000_0000_0000_0000 | suffix),
        model_name,
        provider,
        base_url,
    }
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

    let mut inserted = 0_u64;
    for (index, seed) in SYSTEM_MODEL_SEEDS.iter().enumerate() {
        let sort_order = i32::try_from(index + 1)
            .map_err(|_| storage_overflow())?
            .saturating_mul(10);
        inserted += sqlx::query(INSERT_SEED_SQL)
            .bind(seed.id)
            .bind(seed.model_name)
            .bind(seed.provider)
            .bind(seed.base_url)
            .bind(sort_order)
            .bind(actor_id)
            .execute(&mut *transaction)
            .await
            .map_err(storage("无法补种默认模型"))?
            .rows_affected();
    }

    transaction
        .commit()
        .await
        .map_err(storage("无法提交默认模型补种事务"))?;
    Ok(inserted)
}

fn storage_overflow() -> cloud_domain::AppError {
    cloud_domain::AppError::Internal("默认模型排序超出支持范围".to_owned())
}
