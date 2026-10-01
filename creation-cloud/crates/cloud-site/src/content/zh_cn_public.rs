//! 提供简体中文安全、下载、更新与常见问题内容。

use crate::{FaqItem, PageContent, PageId};

use super::zh_cn::{item, page, section};

pub(super) fn security() -> PageContent {
    page(
        PageId::Security,
        "SSH 隧道、主机密钥与保险库安全｜C-SSH",
        "了解 C-SSH 的 SSH 隧道、主机密钥校验、agent 本地 socket、云端数据边界与客户端加密保险库。",
        "先定义边界，再增加能力",
        "控制面与 SSH 数据面明确分开",
        "连接默认保持端到端直接；云端只接收经过分类、允许上云的数据。",
    )
    .with_sections(vec![
        section(
            "transport",
            "连接与主机安全",
            "关键变更必须对用户可见，陌生资源不会被自动处理。",
            vec![
                item(
                    "SSH",
                    "本地隧道",
                    "客户端主动建立连接，agent 不开放公网监听端口。",
                    "最小暴露",
                ),
                item(
                    "校验",
                    "主机密钥确认",
                    "主机密钥变化需要明确确认，不静默接受。",
                    "可见失败",
                ),
                item(
                    "会话",
                    "不私自结束任务",
                    "不会无授权终止远端 SSH、tmux 或用户进程。",
                    "保留现场",
                ),
            ],
        ),
        section(
            "cloud",
            "云端数据分类",
            "未知字段默认拒绝，敏感资料不以明文进入云端。",
            vec![
                item(
                    "同步",
                    "Host+AI 手动同步",
                    "秘密只以可信客户端加密的不透明密文上传。",
                    "用户确认",
                ),
                item(
                    "保险库",
                    "客户端加密",
                    "服务端仅保存版本化不透明密文与必要非秘密元数据。",
                    "零知识",
                ),
                item(
                    "日志",
                    "脱敏记录",
                    "密码、Token、Cookie、密文正文与 SSH 资料不进入日志。",
                    "最少记录",
                ),
            ],
        ),
    ])
}

pub(super) fn downloads() -> PageContent {
    page(
        PageId::Downloads,
        "下载 C-SSH｜Windows、Mac、Android 与 Linux",
        "下载 C-SSH SSH 终端与服务器运维客户端的 Windows x64／ARM64、M系列Mac、Android与Linux最新版；Linux提供两种架构的deb和AppImage。",
        "DOWNLOADS / DIRECT",
        "下载 C-SSH",
        "0.9.2 仅通过公告告知，请手动下载安装；不推送升级提醒，不强制更新。安装前请阅读数据保全说明。",
    )
    .with_sections(vec![section(
        "builds",
        "选择你的平台",
        "只显示可执行的下载入口；本次未发布的平台保留简短状态。",
        vec![
            item(
                "桌面",
                "Windows",
                "提供可用包型，按钮直达下载。",
                "等待发布数据",
            ),
            item(
                "移动",
                "Android",
                "提供可用包型，按钮直达下载。",
                "等待发布数据",
            ),
            item(
                "规划中",
                "macOS",
                "仅M系列arm64，提供DMG；ad-hoc签名、未经公证，需要手动安装。",
                "本次未发布",
            ),
            item(
                "规划中",
                "iOS",
                "iPhone客户端继续开发，本次不上传或提供下载。",
                "本次未发布",
            ),
        ],
    )])
}

pub(super) fn changelog() -> PageContent {
    page(
        PageId::Changelog,
        "C-SSH 更新日志｜版本与功能变化",
        "查看 C-SSH 每个正式版本的发布时间、功能变化与覆盖平台；下载文件和 SHA256 统一前往下载页。",
        "CHANGELOG / 更新日志",
        "更新日志",
        "记录每个正式版本的功能变化、发布时间与覆盖平台。",
    )
    .with_sections(vec![
        section(
            "latest",
            "最近版本",
            "当前页面不写死可能过期的版本号。",
            vec![item(
                "待接入",
                "发布记录尚未载入",
                "版本服务上线后按发布时间显示已发布版本。",
                "无模拟数据",
            )],
        ),
        section(
            "policy",
            "发布原则",
            "修复通过新版本交付，不原地替换既有公开资产。",
            vec![
                item(
                    "来源",
                    "来源明确",
                    "本站文件与第三方镜像分别标注，不混淆身份。",
                    "可追溯",
                ),
                item(
                    "校验",
                    "哈希可核对",
                    "每项资产展示独立 SHA256 与架构信息。",
                    "不可变",
                ),
                item(
                    "验证",
                    "先验证再发布",
                    "构建、签名与真实功能验证完成后才进入公开记录。",
                    "真实链路",
                ),
            ],
        ),
    ])
}

pub(super) fn faq() -> PageContent {
    page(
        PageId::Faq,
        "SSH 客户端与 agent 常见问题｜C-SSH",
        "解答 C-SSH 的 SSH 连接、常驻 agent、云同步、凭据隐私、安装包校验和移动端定位问题。",
        "常见问题",
        "先把关键边界说清楚",
        "关于连接方式、agent、云同步与下载版本的简明回答。",
    )
    .with_faqs(vec![
        FaqItem::new("Creation Cloud 会代理 SSH 连接吗？", "不会。SSH 数据面保持客户端直连用户服务器，云端只做账号、设备与可选同步等控制面能力。"),
        FaqItem::new("没有安装 agent 还能使用吗？", "普通 SSH 终端和端口映射当前可走原生 SSH；跳板机属于同类架构例外，但本阶段仍延期。持久会话、监控和结构化管理能力依赖 agent。"),
        FaqItem::new("主机地址和私钥会同步到云端吗？", "主机名称、地址、端口、标签、状态、连接设置和凭据只以客户端加密密文手动同步；AI Key/API/模型绑定遵循相同的客户端保护边界。known_hosts、终端内容和命令历史不上云。"),
        FaqItem::new("保险库密码和账号密码相同吗？", "不同。账号密码只负责登录；保险库密码只在可信客户端派生加密密钥，不上传服务端。"),
        FaqItem::new("如何确认下载文件未被替换？", "正式下载条目会展示平台、架构、文件大小与 SHA256；请在安装前核对。"),
        FaqItem::new("移动端是桌面端的完整复制吗？", "不是。Android 定位为移动伴侣，优先覆盖查看、轻量操作和与桌面工作流衔接的场景。"),
    ])
}
