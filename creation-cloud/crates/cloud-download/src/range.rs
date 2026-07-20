//! 解析单段 HTTP bytes Range，并拒绝多段与越界请求。

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ByteRange {
    Full,
    Partial { start: u64, end: u64 },
}

impl ByteRange {
    pub(crate) fn parse(value: Option<&str>, size: u64) -> Result<Self, ()> {
        let Some(value) = value else {
            return Ok(Self::Full);
        };
        let value = value.strip_prefix("bytes=").ok_or(())?;
        if value.contains(',') || size == 0 {
            return Err(());
        }
        let (start, end) = value.split_once('-').ok_or(())?;
        if start.is_empty() {
            let suffix = end.parse::<u64>().map_err(|_| ())?;
            if suffix == 0 {
                return Err(());
            }
            let start = size.saturating_sub(suffix.min(size));
            return Ok(Self::Partial {
                start,
                end: size - 1,
            });
        }

        let start = start.parse::<u64>().map_err(|_| ())?;
        if start >= size {
            return Err(());
        }
        let end = if end.is_empty() {
            size - 1
        } else {
            end.parse::<u64>().map_err(|_| ())?.min(size - 1)
        };
        if end < start {
            return Err(());
        }
        Ok(Self::Partial { start, end })
    }

    #[must_use]
    pub(crate) const fn bounds(self, size: u64) -> (u64, u64, bool) {
        match self {
            Self::Full => (0, size.saturating_sub(1), false),
            Self::Partial { start, end } => (start, end, true),
        }
    }
}
