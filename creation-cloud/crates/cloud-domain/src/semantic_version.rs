//! Creation Cloud 发布与更新检查共享的语义版本值对象。

use std::cmp::Ordering;

pub const MAX_SEMANTIC_VERSION_LENGTH: usize = 64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticVersion {
    major: NumericIdentifier,
    minor: NumericIdentifier,
    patch: NumericIdentifier,
    prerelease: Option<Vec<PrereleaseIdentifier>>,
}

impl SemanticVersion {
    /// 解析不带 `v` 前缀的 SemVer 2.0.0 文本。
    fn parse(value: &str) -> Result<Self, ()> {
        if value.is_empty() || value.len() > MAX_SEMANTIC_VERSION_LENGTH || !value.is_ascii() {
            return Err(());
        }
        let (without_build, build) = match value.split_once('+') {
            Some((version, build)) => {
                validate_dot_identifiers(build, false)?;
                (version, Some(build))
            }
            None => (value, None),
        };
        if build.is_some_and(|value| value.contains('+')) {
            return Err(());
        }
        let (core, prerelease) = match without_build.split_once('-') {
            Some((core, prerelease)) => {
                let identifiers = parse_prerelease(prerelease)?;
                (core, Some(identifiers))
            }
            None => (without_build, None),
        };
        let mut core = core.split('.');
        let major = NumericIdentifier::parse(core.next().ok_or(())?)?;
        let minor = NumericIdentifier::parse(core.next().ok_or(())?)?;
        let patch = NumericIdentifier::parse(core.next().ok_or(())?)?;
        if core.next().is_some() {
            return Err(());
        }
        Ok(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

/// 接受可选的小写 `v` 前缀并返回不带前缀的规范文本和值对象。
pub fn normalize_semantic_version(value: &str) -> Option<(String, SemanticVersion)> {
    let normalized = value.strip_prefix('v').unwrap_or(value);
    SemanticVersion::parse(normalized)
        .ok()
        .map(|version| (normalized.to_owned(), version))
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| {
                compare_prerelease(self.prerelease.as_deref(), other.prerelease.as_deref())
            })
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NumericIdentifier(String);

impl NumericIdentifier {
    fn parse(value: &str) -> Result<Self, ()> {
        if value.is_empty()
            || !value.bytes().all(|byte| byte.is_ascii_digit())
            || (value.len() > 1 && value.starts_with('0'))
        {
            return Err(());
        }
        Ok(Self(value.to_owned()))
    }
}

impl Ord for NumericIdentifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&other.0.len())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl PartialOrd for NumericIdentifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PrereleaseIdentifier {
    Numeric(NumericIdentifier),
    Text(String),
}

fn parse_prerelease(value: &str) -> Result<Vec<PrereleaseIdentifier>, ()> {
    validate_dot_identifiers(value, true)?;
    value
        .split('.')
        .map(|identifier| {
            if identifier.bytes().all(|byte| byte.is_ascii_digit()) {
                NumericIdentifier::parse(identifier).map(PrereleaseIdentifier::Numeric)
            } else {
                Ok(PrereleaseIdentifier::Text(identifier.to_owned()))
            }
        })
        .collect()
}

fn validate_dot_identifiers(value: &str, prerelease: bool) -> Result<(), ()> {
    if value.is_empty() {
        return Err(());
    }
    for identifier in value.split('.') {
        if identifier.is_empty()
            || !identifier
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || (prerelease
                && identifier.len() > 1
                && identifier.starts_with('0')
                && identifier.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return Err(());
        }
    }
    Ok(())
}

fn compare_prerelease(
    left: Option<&[PrereleaseIdentifier]>,
    right: Option<&[PrereleaseIdentifier]>,
) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => {
            for (left, right) in left.iter().zip(right) {
                let ordering = match (left, right) {
                    (PrereleaseIdentifier::Numeric(left), PrereleaseIdentifier::Numeric(right)) => {
                        left.cmp(right)
                    }
                    (PrereleaseIdentifier::Numeric(_), PrereleaseIdentifier::Text(_)) => {
                        Ordering::Less
                    }
                    (PrereleaseIdentifier::Text(_), PrereleaseIdentifier::Numeric(_)) => {
                        Ordering::Greater
                    }
                    (PrereleaseIdentifier::Text(left), PrereleaseIdentifier::Text(right)) => {
                        left.cmp(right)
                    }
                };
                if ordering != Ordering::Equal {
                    return ordering;
                }
            }
            left.len().cmp(&right.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_optional_v_and_orders_prereleases() {
        let (normalized, stable) =
            normalize_semantic_version("v7.1.0").expect("带 v 的有效版本应被规范化");
        let (_, beta_ten) =
            normalize_semantic_version("7.1.0-beta.10").expect("有效预发布版本应可解析");
        let (_, beta_two) =
            normalize_semantic_version("7.1.0-beta.2").expect("有效预发布版本应可解析");
        assert_eq!(normalized, "7.1.0");
        assert!(stable > beta_ten);
        assert!(beta_ten > beta_two);
    }

    #[test]
    fn rejects_non_semver_and_noncanonical_numeric_identifiers() {
        for value in ["smoke-run", "7.0", "07.0.0", "7.0.0-beta.01", "V7.0.0"] {
            assert!(
                normalize_semantic_version(value).is_none(),
                "{value} 不应被接受"
            );
        }
    }
}
