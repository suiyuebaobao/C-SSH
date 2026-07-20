//! 集中处理管理端响应中的邮箱脱敏，禁止任何查询动作直接返回原文邮箱。

pub(crate) fn email(value: &str) -> String {
    let Some((local, domain)) = value.split_once('@') else {
        return "***".to_owned();
    };
    let local = masked_label(local);
    let domain = match domain.rsplit_once('.') {
        Some((name, suffix)) if !name.is_empty() && !suffix.is_empty() => {
            format!("{}.{}", masked_label(name), suffix)
        }
        _ => masked_label(domain),
    };
    format!("{local}@{domain}")
}

fn masked_label(value: &str) -> String {
    value
        .chars()
        .next()
        .map_or_else(|| "***".to_owned(), |first| format!("{first}***"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_local_and_domain_labels() {
        let masked = email("admin@example.com");
        assert_eq!(masked, "a***@e***.com");
        assert!(!masked.contains("admin"));
        assert!(!masked.contains("example"));
    }
}
