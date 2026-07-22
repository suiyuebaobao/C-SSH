//! 把环境变量解析为受限整数与时长，不接受缺失 Unicode 或范围外数值。

use std::{ffi::OsString, ops::RangeInclusive, time::Duration};

use anyhow::{Context, Result, bail};

pub(super) fn duration_from(
    lookup: &mut impl FnMut(&str) -> Option<OsString>,
    name: &'static str,
    default: u64,
    range: RangeInclusive<u64>,
    unit_seconds: u64,
) -> Result<Duration> {
    let value = number_from(lookup, name, default, range)?;
    let seconds = value
        .checked_mul(unit_seconds)
        .with_context(|| format!("{name} 换算后超出范围"))?;
    Ok(Duration::from_secs(seconds))
}

pub(super) fn number_from(
    lookup: &mut impl FnMut(&str) -> Option<OsString>,
    name: &'static str,
    default: u64,
    range: RangeInclusive<u64>,
) -> Result<u64> {
    let Some(raw) = lookup(name) else {
        return Ok(default);
    };
    let raw = raw
        .into_string()
        .map_err(|_| anyhow::anyhow!("{name} 必须是有效 UTF-8 整数"))?;
    let value = raw
        .parse::<u64>()
        .with_context(|| format!("{name} 必须是整数"))?;
    if !range.contains(&value) {
        bail!("{name} 超出允许范围");
    }
    Ok(value)
}
