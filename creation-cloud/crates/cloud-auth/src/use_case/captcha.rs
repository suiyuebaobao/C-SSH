//! 生成短期一次性认证 CAPTCHA，并只持久化用途绑定 HMAC 摘要。

use chrono::Utc;
use cloud_domain::{AppError, AppResult};
use cloud_store::PgPool;
use uuid::Uuid;

use crate::{
    captcha::{self, CaptchaPurpose},
    repository, verification,
};

pub(crate) struct IssuedCaptcha {
    pub challenge_id: Uuid,
    pub expires_at: chrono::DateTime<Utc>,
    pub svg: String,
}

pub(crate) async fn issue(
    pool: &PgPool,
    key: &[u8],
    purpose: CaptchaPurpose,
) -> AppResult<IssuedCaptcha> {
    if key.len() < 32 {
        return Err(AppError::Unavailable("图形验证码密钥尚未就绪".to_owned()));
    }
    let challenge_id = Uuid::now_v7();
    let code = captcha::issue_code();
    let code_digest = verification::captcha_digest(key, challenge_id, purpose, &code);
    let expires_at = Utc::now() + chrono::Duration::minutes(captcha::TTL_MINUTES);
    let svg = captcha::render_svg(&code);
    repository::captcha::insert(pool, challenge_id, purpose, &code_digest, expires_at).await?;
    Ok(IssuedCaptcha {
        challenge_id,
        expires_at,
        svg,
    })
}
