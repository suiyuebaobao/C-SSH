//! 定义网站支持的语言并收敛语言代码解析规则。

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Locale {
    #[default]
    ZhCn,
    En,
}

impl Locale {
    #[must_use]
    pub fn from_code(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some(value) if value.eq_ignore_ascii_case("en") => Self::En,
            Some(value) if value.eq_ignore_ascii_case("en-us") => Self::En,
            _ => Self::ZhCn,
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    #[must_use]
    pub const fn alternate(self) -> Self {
        match self {
            Self::ZhCn => Self::En,
            Self::En => Self::ZhCn,
        }
    }
}
