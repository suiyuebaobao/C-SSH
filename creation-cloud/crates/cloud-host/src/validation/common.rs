fn validate_metadata(metadata: &HostMetadataInput) -> AppResult<()> {
    validate_address(&metadata.address)?;
    if metadata.port == 0 {
        return Err(AppError::Validation(
            "port must be between 1 and 65535".to_owned(),
        ));
    }
    validate_text(&metadata.name, 128, "name")?;
    validate_text(&metadata.platform, 32, "platform")?;
    if metadata.tags.len() > 32 {
        return Err(AppError::Validation("tags 不能超过 32 项".to_owned()));
    }
    let mut tags = HashSet::with_capacity(metadata.tags.len());
    for tag in &metadata.tags {
        validate_text(tag, 48, "tag")?;
        if !tags.insert(tag) {
            return Err(AppError::Validation("tags 不得重复".to_owned()));
        }
    }
    Ok(())
}

fn validate_address(address: &str) -> AppResult<()> {
    address
        .parse::<IpAddr>()
        .map(|_| ())
        .map_err(|_| AppError::Validation("address 必须是纯 IPv4 或 IPv6 地址".to_owned()))
}

fn validate_text(value: &str, max_chars: usize, field: &str) -> AppResult<()> {
    let count = value.chars().count();
    if count == 0
        || count > max_chars
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation(format!("{field} 长度或字符不合法")));
    }
    Ok(())
}

fn decode_ciphertext(value: Option<&Option<String>>) -> AppResult<Option<Option<Vec<u8>>>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(value) = value.as_deref() else {
        return Ok(Some(None));
    };
    opaque::decode_required(value, "ciphertext", MAX_CIPHERTEXT_BYTES)
        .map(|value| Some(Some(value)))
}

fn generation(value: i64) -> AppResult<()> {
    if value <= 0 {
        Err(AppError::Validation(
            "sync_generation 必须大于 0".to_owned(),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn protection_version(epoch: i64, revision: i64, configured: bool) -> AppResult<()> {
    let valid = if configured {
        epoch > 0 && revision > 0
    } else {
        epoch >= 0 && revision >= 0 && ((epoch == 0) == (revision == 0))
    };
    if valid {
        Ok(())
    } else {
        Err(AppError::Validation(
            "protection_epoch/protection_revision 组合不合法".to_owned(),
        ))
    }
}

pub(crate) fn verify_reset_challenge(
    request: &VerifyProtectionResetChallengeRequest,
) -> AppResult<()> {
    require_uuid(request.challenge_id, "challenge_id")?;
    if request.code.len() == 6 && request.code.bytes().all(|value| value.is_ascii_digit()) {
        Ok(())
    } else {
        Err(AppError::Validation("邮箱验证码必须是 6 位数字".to_owned()))
    }
}

fn positive_expected(value: Option<i64>) -> AppResult<i64> {
    value
        .filter(|revision| *revision > 0)
        .ok_or_else(|| AppError::Validation("expected_revision 必须大于 0".to_owned()))
}

fn require_uuid(value: Uuid, field: &str) -> AppResult<()> {
    if value.is_nil() {
        Err(AppError::Validation(format!("{field} 不能为空")))
    } else {
        Ok(())
    }
}

pub(crate) fn host_id(host_id: Uuid) -> AppResult<()> {
    require_uuid(host_id, "host_id")
}
