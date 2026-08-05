//! 生成不含明文字符节点的短期认证图形验证码 SVG。

use std::fmt::Write;

use rand::Rng;

pub(crate) const CODE_LENGTH: usize = 6;
pub(crate) const TTL_MINUTES: i64 = 5;
pub(crate) const MAX_ATTEMPTS: i32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptchaPurpose {
    Register,
    Login,
    AdminLogin,
    PasswordReset,
}

impl CaptchaPurpose {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Login => "login",
            Self::AdminLogin => "admin_login",
            Self::PasswordReset => "password_reset",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "register" => Some(Self::Register),
            "login" => Some(Self::Login),
            "admin_login" => Some(Self::AdminLogin),
            "password_reset" => Some(Self::PasswordReset),
            _ => None,
        }
    }
}

const SEGMENTS: [[bool; 7]; 10] = [
    [true, true, true, false, true, true, true],
    [false, false, true, false, false, true, false],
    [true, false, true, true, true, false, true],
    [true, false, true, true, false, true, true],
    [false, true, true, true, false, true, false],
    [true, true, false, true, false, true, true],
    [true, true, false, true, true, true, true],
    [true, false, true, false, false, true, false],
    [true, true, true, true, true, true, true],
    [true, true, true, true, false, true, true],
];

pub(crate) fn issue_code() -> String {
    format!(
        "{:0width$}",
        rand::rng().random_range(0_u32..1_000_000),
        width = CODE_LENGTH
    )
}

pub(crate) fn render_svg(code: &str) -> String {
    let mut rng = rand::rng();
    let mut svg = String::from(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="240" height="80" viewBox="0 0 240 80" role="img" aria-label="Authentication CAPTCHA"><rect width="240" height="80" rx="8" fill="#f3efe6"/>"##,
    );
    for _ in 0..9 {
        let x1 = rng.random_range(0..240);
        let y1 = rng.random_range(4..76);
        let x2 = rng.random_range(0..240);
        let y2 = rng.random_range(4..76);
        let opacity = rng.random_range(12..31);
        write!(
            svg,
            r##"<path d="M{x1} {y1} L{x2} {y2}" stroke="#ef3b1f" stroke-width="1" opacity="0.{opacity}"/>"##
        )
        .expect("writing to String cannot fail");
    }
    for (index, digit) in code.bytes().enumerate() {
        let value = usize::from(digit.saturating_sub(b'0')).min(9);
        let x = 18 + i32::try_from(index).unwrap_or_default() * 36 + rng.random_range(-2..=2);
        let y = 10 + rng.random_range(-2..=2);
        let angle = rng.random_range(-7..=7);
        write!(
            svg,
            r##"<g fill="#111318" transform="rotate({angle} {} 40)">"##,
            x + 14
        )
        .expect("writing to String cannot fail");
        for (segment, visible) in SEGMENTS[value].iter().enumerate() {
            if !visible {
                continue;
            }
            let (segment_x, segment_y, width, height) = match segment {
                0 => (x + 5, y, 19, 5),
                1 => (x, y + 5, 5, 23),
                2 => (x + 24, y + 5, 5, 23),
                3 => (x + 5, y + 28, 19, 5),
                4 => (x, y + 33, 5, 23),
                5 => (x + 24, y + 33, 5, 23),
                _ => (x + 5, y + 56, 19, 5),
            };
            write!(
                svg,
                r#"<rect x="{segment_x}" y="{segment_y}" width="{width}" height="{height}" rx="2"/>"#
            )
            .expect("writing to String cannot fail");
        }
        svg.push_str("</g>");
    }
    for _ in 0..32 {
        let cx = rng.random_range(2..238);
        let cy = rng.random_range(2..78);
        let radius = rng.random_range(1..=2);
        write!(
            svg,
            r##"<circle cx="{cx}" cy="{cy}" r="{radius}" fill="#ef3b1f" opacity="0.35"/>"##
        )
        .expect("writing to String cannot fail");
    }
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_code_is_six_ascii_digits() {
        for _ in 0..100 {
            let code = issue_code();
            assert_eq!(code.len(), CODE_LENGTH);
            assert!(code.bytes().all(|byte| byte.is_ascii_digit()));
        }
    }

    #[test]
    fn svg_contains_no_plaintext_code_or_text_node() {
        let code = "123456";
        let svg = render_svg(code);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(!svg.contains(code));
        assert!(!svg.contains("<text"));
        assert!(!svg.contains("<script"));
    }
}
