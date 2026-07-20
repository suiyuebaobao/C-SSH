//! 提供官网反馈双渠道的简体中文元数据与安全说明。

use crate::{PageContent, PageId};

use super::zh_cn::{item, page, section};

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Feedback,
        "Creation-SSH 问题反馈与建议｜官网工单和 GitHub Issues",
        "通过 Creation-SSH 官网工单或 GitHub Issues 提交问题与建议，并按安全说明提供平台、版本、复现步骤和脱敏信息。",
        "FEEDBACK / TWO CHANNELS",
        "把问题送到真正有人处理的地方",
        "登录用户可以把反馈直接提交到官网后台；适合公开讨论的问题也可以进入 GitHub Issues。",
    )
    .with_sections(vec![
        section(
            "channels",
            "两个渠道，各有明确边界",
            "官网工单与 GitHub Issues 都是真实入口，不用假表单或占位链接冒充受理。",
            vec![
                item(
                    "官网",
                    "登录后提交工单",
                    "反馈通过会话和 CSRF 校验写入 PostgreSQL，成功后返回唯一编号并进入管理后台。",
                    "适合账号内跟踪",
                ),
                item(
                    "GitHub",
                    "公开 Issues",
                    "适合可以公开复现、讨论和协作的问题；提交时需要登录 GitHub 账号。",
                    "适合公开协作",
                ),
            ],
        ),
        section(
            "prepare",
            "提交前准备最小可复现信息",
            "清楚的信息比完整日志更有价值，也更安全。",
            vec![
                item(
                    "环境",
                    "平台与版本",
                    "说明 Windows、Linux、Android、macOS、iOS、Cloud 或 agent，以及实际应用版本。",
                    "不要猜测版本",
                ),
                item(
                    "步骤",
                    "预期与实际结果",
                    "用最少步骤描述如何出现问题，并分别说明你预期看到什么、实际发生了什么。",
                    "可以稳定复现",
                ),
                item(
                    "安全",
                    "只提交脱敏文本",
                    "不要提交密码、私钥、Token、真实主机地址、Cookie 或完整敏感日志；当前不接收附件。",
                    "发现敏感内容会拒绝或脱敏",
                ),
            ],
        ),
    ])
}
