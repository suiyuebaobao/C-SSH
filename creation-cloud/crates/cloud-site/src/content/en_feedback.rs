//! 提供官网反馈双渠道的英文元数据与安全说明。

use crate::{PageContent, PageId};

use super::en::{item, page, section};

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Feedback,
        "Creation-SSH feedback | Website tickets and GitHub Issues",
        "Report Creation-SSH problems through a website ticket or GitHub Issues with platform, version, reproduction steps, and safely redacted context.",
        "FEEDBACK / TWO CHANNELS",
        "Send each problem somewhere it will actually be handled",
        "Signed-in users can submit directly to the website back office, while public discussions can continue in GitHub Issues.",
    )
    .with_sections(vec![
        section(
            "channels",
            "Two channels with explicit boundaries",
            "Both the website ticket and GitHub Issues are real destinations, never placeholder forms.",
            vec![
                item(
                    "Website",
                    "Submit after signing in",
                    "The ticket passes session and CSRF checks, persists in PostgreSQL, returns a unique ID, and enters the admin workflow.",
                    "Account-scoped tracking",
                ),
                item(
                    "GitHub",
                    "Public Issues",
                    "Use this channel for problems that can be reproduced, discussed, and collaborated on in public. A GitHub account is required.",
                    "Public collaboration",
                ),
            ],
        ),
        section(
            "prepare",
            "Prepare the smallest reproducible report",
            "Clear context is more useful and safer than a complete log dump.",
            vec![
                item(
                    "Environment",
                    "Platform and version",
                    "Identify Windows, Linux, Android, macOS, iOS, Cloud, or agent and provide the actual application version.",
                    "Do not guess the version",
                ),
                item(
                    "Steps",
                    "Expected and actual results",
                    "Describe the shortest reproduction path, then separate what you expected from what actually happened.",
                    "Make it reproducible",
                ),
                item(
                    "Safety",
                    "Submit redacted text only",
                    "Never include passwords, private keys, tokens, real host addresses, cookies, or full sensitive logs. Attachments are not accepted yet.",
                    "Sensitive content is rejected or redacted",
                ),
            ],
        ),
    ])
}
