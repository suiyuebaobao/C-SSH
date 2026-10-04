//! 集中处理设备等跨账号管理响应中的邮箱脱敏。

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
