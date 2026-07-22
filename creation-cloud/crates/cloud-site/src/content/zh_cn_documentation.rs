//! 提供简体中文文档入口、发布状态、实操指南与安全参考内容。

use crate::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationPlatform, DocumentationScreenshot, DocumentationSection,
    PageContent, PageId,
};

use super::zh_cn::{action, page};

const RELEASE_HREF: &str = "https://github.com/suiyuebaobao/C-SSH/releases/tag/v0.6.16";

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Documentation,
        "Creation-SSH 使用文档｜安装、连接与实操指南",
        "从可信下载、添加主机到 agent 部署、持久终端、监控、文件与 AI 的可验证 Creation-SSH 使用文档。",
        "DOCS / TASK GUIDES",
        "从第一台主机开始使用 Creation-SSH",
        "文档和实操指南已经合并。按目录完成一个真实任务，并用每章的预期结果判断链路是否可用。",
    )
    .with_actions(vec![
        action("查看 v0.6.16 发布页", RELEASE_HREF, "button button-secondary"),
        action("下载客户端", "/downloads", "button button-primary"),
    ])
    .with_documentation_page(documentation())
}

fn documentation() -> DocumentationContent {
    DocumentationContent {
        release_label: "当前公开发布",
        release_version: "v0.6.16",
        release_date: "2026-07-18",
        release_href: RELEASE_HREF,
        release_action_label: "打开 Release",
        index_label: "文档目录",
        mobile_index_label: "展开文档目录",
        search_label: "筛选本页标题",
        search_placeholder: "例如：主机、终端、监控",
        search_help: "只筛选当前页面已经加载的标题，不搜索正文，也不会跳转到站外。",
        search_empty: "当前页没有匹配的标题。",
        status: DocumentationNotice::new(
            "开始前",
            "先验证来源与主机身份",
            "仅从 Creation-SSH 公开 Release 获取资产。SHA256 缺失或不匹配、主机密钥变化、架构不受支持，任一情况都应立即停止。",
        ),
        groups: groups(),
        platform_code: "00 / PLATFORM MATRIX",
        platform_title: "选择与你的平台和架构匹配的发布资产",
        platform_lead: "v0.6.16 共七个正式安装或发布资产。Windows、Linux、Android 已交付；macOS 与 iOS 仍处于规划中。",
        platforms: platforms(),
        tutorials: super::zh_cn_tutorials::content(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "普通 PTY 与持久化终端是两条不同路径",
            lead: "图中是直连普通 PTY；切换到持久化终端后，才由客户端、agent 与 tmux 协作提供可重连会话。",
            src: "/static/img/product-terminal.png",
            alt: "Creation-SSH 脱敏演示终端，显示示例服务器和普通 SSH PTY",
            caption: "脱敏演示图：地址使用 RFC 5737 示例值；只说明界面路径，不作为持久化会话、发布状态或 no-mock 证据。",
            width: 1650,
            height: 1080,
        },
        final_code: "NEXT / KEEP EVIDENCE",
        final_title: "遇到问题，先保留现场再反馈",
        final_body: "不要静默接受主机密钥变化，不要结束未经授权的远端会话，也不要在反馈中提交真实地址、密码、私钥、Token 或完整敏感日志。",
    }
}

fn groups() -> Vec<DocumentationGroup> {
    vec![
        group(
            "快速开始",
            vec![
                link("platform-matrix", "00", "平台与发布状态"),
                link("getting-started", "01", "下载、校验与安装"),
            ],
        ),
        group(
            "实操指南",
            vec![
                link("add-host", "02", "添加第一台主机"),
                link("deploy-agent", "03", "部署或修复 agent"),
                link("persistent-terminal", "04", "建立可重连终端"),
                link("monitoring", "05", "启用监控并看历史"),
                link("files", "06", "浏览并传输文件"),
                link("ai-assistant", "07", "配置并运行 AI 助手"),
            ],
        ),
        group(
            "参考与安全",
            vec![
                link("port-forwarding", "08", "本地 SSH 转发"),
                link("cloud-security", "09", "Cloud 与数据边界"),
                link("troubleshooting", "10", "安全停止条件"),
            ],
        ),
    ]
}

fn platforms() -> Vec<DocumentationPlatform> {
    vec![
        DocumentationPlatform::released(
            "W",
            "Windows",
            "桌面主线",
            "v0.6.16 已发布",
            "Creation-SSH_0.6.16_x64-setup.exe · Creation-SSH_0.6.16_x64_en-US.msi · Creation-SSH_0.6.16_portable-Windows-x64.zip",
            "选择 NSIS、MSI 或便携包之一；不要同时安装多个变体。",
            RELEASE_HREF,
        ),
        DocumentationPlatform::released(
            "L",
            "Linux",
            "独立桌面",
            "v0.6.16 已发布",
            "Creation-SSH_0.6.16_linux-amd64.deb · Creation-SSH_0.6.16_linux-x86_64.AppImage",
            "Debian 系选择 deb；其他 x86_64 桌面可按发行版策略使用 AppImage。",
            RELEASE_HREF,
        ),
        DocumentationPlatform::released(
            "A",
            "Android",
            "移动伴侣",
            "v0.6.16 已发布",
            "C-SSH_0.6.16_android-arm64.apk · C-SSH_0.6.16_android-arm64.aab",
            "普通用户使用已签名 APK；AAB 面向应用商店分发，不是直接安装包。",
            RELEASE_HREF,
        ),
        DocumentationPlatform::planned(
            "m",
            "macOS",
            "未来桌面",
            "规划中，无下载",
            "尚无可交付客户端或安装包。",
        ),
        DocumentationPlatform::planned(
            "i",
            "iOS",
            "未来移动伴侣",
            "规划中，无下载",
            "尚未完成开发与真机验证。",
        ),
    ]
}

fn sections() -> Vec<DocumentationSection> {
    vec![
        section(
            "getting-started",
            "01 / QUICK START",
            "下载、校验与安装",
            "先建立可信的软件来源，再开始连接服务器。",
            vec![
                text(
                    "SOURCE",
                    "只认公开 Release",
                    "打开 v0.6.16 发布页，按平台矩阵选择一个正式资产；不要从聊天附件、网盘转存或不明镜像安装。",
                    true,
                ),
                command(
                    "SHA256",
                    "逐字核对校验值",
                    "计算下载文件的 SHA256，并与 Release notes 中同名资产逐字比较。缺少校验值或任一字符不一致都应停止。",
                    "Windows: Get-FileHash .\\<asset> -Algorithm SHA256\nLinux:   sha256sum ./<asset>",
                    true,
                ),
                text(
                    "INSTALL",
                    "只安装匹配变体",
                    "Windows 选择 NSIS、MSI 或便携包之一；Linux 选择 deb 或 AppImage；Android 普通安装使用已签名 APK。macOS 与 iOS 没有下载。",
                    false,
                ),
            ],
        ),
        section(
            "port-forwarding",
            "08 / PORT FORWARDING",
            "使用原生 SSH 本地转发",
            "端口映射是保留的纯 SSH 例外，不依赖 agent。",
            vec![command(
                "LOOPBACK",
                "默认仅绑定本机回环",
                "把服务器可访问的目标端口映射到本机 127.0.0.1；只有明确理解暴露面时才调整监听地址。",
                "127.0.0.1:<local-port> -> <remote-host>:<remote-port>",
                true,
            )],
        ),
        section(
            "cloud-security",
            "09 / CLOUD & SECURITY",
            "Cloud 账号可选，数据面仍走 SSH",
            "当前本地实现尚未部署，也未连接线上服务；页面存在不代表云端能力已经交付。",
            vec![
                text(
                    "OPTIONAL",
                    "无需 Cloud 账号也可管理本地主机",
                    "SSH 连接、普通终端和本地工作流不以 Cloud 登录为前提。Cloud 只规划承载账号、设备、同步、模型、保险库信封、版本与下载控制面。",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud 不代理 SSH 数据面",
                    "私钥、密码和明文敏感资料不上云；只有专项方案定义的客户端本地加密 vault 信封可存储。",
                    true,
                ),
            ],
        ),
        section(
            "troubleshooting",
            "10 / TROUBLESHOOTING",
            "遇到异常时安全停止",
            "先保住身份与远端工作，再定位网络、架构、资源和权限。",
            vec![
                text(
                    "HOST KEY",
                    "主机密钥变化",
                    "停止连接，使用可信渠道核对新指纹与变更原因；不要删除 known_hosts 记录来跳过确认。",
                    true,
                ),
                text(
                    "DEPLOY",
                    "架构或配对资源缺失",
                    "重新执行真实 uname -m 探测；SQLite 中的末次架构只作记录。缺少匹配 agent 或 tmux 时不要上传另一架构，也不要两套全传。",
                    true,
                ),
                text(
                    "SESSION",
                    "重连没有恢复",
                    "确认使用的是持久化终端而非普通 PTY，并检查 agent 与 tmux 归属；未经授权不要 kill 远端会话或清理未知 socket。",
                    true,
                ),
            ],
        ),
    ]
}

fn group(title: &'static str, links: Vec<DocumentationLink>) -> DocumentationGroup {
    DocumentationGroup::new(title, links)
}

const fn link(anchor: &'static str, code: &'static str, title: &'static str) -> DocumentationLink {
    DocumentationLink::new(anchor, code, title)
}

fn section(
    anchor: &'static str,
    code: &'static str,
    title: &'static str,
    lead: &'static str,
    items: Vec<DocumentationItem>,
) -> DocumentationSection {
    DocumentationSection::new(anchor, code, title, lead, items)
}

const fn text(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::text(badge, title, body, caution)
}

const fn command(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    value: &'static str,
    caution: bool,
) -> DocumentationItem {
    DocumentationItem::command(badge, title, body, value, caution)
}
