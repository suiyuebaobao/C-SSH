//! 解析单段 HTTP bytes Range，并拒绝多段与越界请求。

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidByteRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteRange {
    Full,
    Partial { start: u64, end: u64 },
}

impl ByteRange {
    pub fn parse(value: Option<&str>, size: u64) -> Result<Self, InvalidByteRange> {
        let Some(value) = value else {
            return Ok(Self::Full);
        };
        let value = value.strip_prefix("bytes=").ok_or(InvalidByteRange)?;
        if value.contains(',') || size == 0 {
            return Err(InvalidByteRange);
        }
        let (start, end) = value.split_once('-').ok_or(InvalidByteRange)?;
        if start.is_empty() {
            let suffix = end.parse::<u64>().map_err(|_| InvalidByteRange)?;
            if suffix == 0 {
                return Err(InvalidByteRange);
            }
            let start = size.saturating_sub(suffix.min(size));
            return Ok(Self::Partial {
                start,
                end: size - 1,
            });
        }

        let start = start.parse::<u64>().map_err(|_| InvalidByteRange)?;
        if start >= size {
            return Err(InvalidByteRange);
        }
        let end = if end.is_empty() {
            size - 1
        } else {
            end.parse::<u64>()
                .map_err(|_| InvalidByteRange)?
                .min(size - 1)
        };
        if end < start {
            return Err(InvalidByteRange);
        }
        Ok(Self::Partial { start, end })
    }

    #[must_use]
    pub const fn bounds(self, size: u64) -> (u64, u64, bool) {
        match self {
            Self::Full => (0, size.saturating_sub(1), false),
            Self::Partial { start, end } => (start, end, true),
        }
    }
}
