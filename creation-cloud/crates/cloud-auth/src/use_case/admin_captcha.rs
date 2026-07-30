//! 生成短期一次性管理员登录 CAPTCHA，并只持久化域隔离 HMAC 摘要。

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{captcha, repository, verification};

pub(crate) struct IssuedAdminCaptcha {
    pub challenge_id: Uuid,
    pub expires_at: chrono::DateTime<Utc>,
    pub svg: String,
}

pub(crate) async fn issue(pool: &PgPool, key: &[u8]) -> AppResult<IssuedAdminCaptcha> {
    if key.len() < 32 {
        return Err(AppError::Unavailable(
            "管理员图形验证码密钥尚未就绪".to_owned(),
        ));
    }
    let challenge_id = Uuid::now_v7();
    let code = captcha::issue_code();
    let code_digest = verification::captcha_digest(key, challenge_id, &code);
    let expires_at = Utc::now() + chrono::Duration::minutes(captcha::TTL_MINUTES);
    let svg = captcha::render_svg(&code);
    repository::captcha::insert(pool, challenge_id, &code_digest, expires_at).await?;
    Ok(IssuedAdminCaptcha {
        challenge_id,
        expires_at,
        svg,
    })
}
