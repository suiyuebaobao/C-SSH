//! 提供简体中文入门文档、平台发布状态与当前产品边界。

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
        "安装与首次连接文档 | Creation-SSH",
        "从可信下载、首次主机密钥确认到 agent 部署、持久化终端重连验证的 Creation-SSH 双语入门文档。",
        "DOCS / GETTING STARTED",
        "安装、首次连接与持续运维",
        "这是一条从可信安装包到可重连工作现场的完整路径。先确认发布资产与主机身份，再启用 agent 能力。",
    )
    .with_actions(vec![
        action("查看 v0.6.16 发布页", RELEASE_HREF, "button button-primary"),
        action("继续看教程", "/tutorials", "button button-secondary"),
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
        index_label: "本页文档目录",
        mobile_index_label: "展开本页目录",
        search_label: "筛选本页章节标题",
        search_placeholder: "例如：终端、监控、Cloud",
        search_help: "只筛选当前页面目录中的章节标题，不搜索正文，也不会跳转到站外。",
        search_empty: "当前页没有匹配的章节标题。",
        status: DocumentationNotice::new(
            "STATUS / SAFETY",
            "先验证来源与主机身份，再执行任何安装或部署",
            "仅从 Creation-SSH 公开 Release 获取资产。SHA256 缺失或不匹配、主机密钥变化、架构不受支持，任一情况都应立即停止；不要用猜测绕过安全检查。",
        ),
        groups: groups(),
        platform_code: "00 / PLATFORM MATRIX",
        platform_title: "选择与你的平台和架构匹配的发布资产",
        platform_lead: "v0.6.16 发布于 2026-07-18，共七个正式安装/发布资产（不含 GitHub 自动生成的源码归档）。Windows、Linux、Android 有交付；macOS 与 iOS 仅处于规划中。",
        platforms: platforms(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "终端界面明确区分普通 PTY 与持久化会话",
            lead: "图中选中的是直连普通 PTY；切换到持久化终端后，才由客户端、agent 与 tmux 协作提供可重连会话。",
            src: "/static/img/product-terminal.png",
            alt: "Creation-SSH 脱敏演示终端，显示示例服务器和当前选中的普通 SSH PTY",
            caption: "脱敏演示图：当前显示普通终端（直连 PTY），地址使用 RFC 5737 示例值；仅说明界面与路径区分，不作为持久化会话、发布状态或 no-mock 验证证据。",
            width: 1650,
            height: 1080,
        },
        final_code: "NEXT / OPERATE WITH EVIDENCE",
        final_title: "完成重连验证后，再逐步开启监控、文件与系统能力",
        final_body: "保持失败可见：不要静默接受主机密钥变化，不要结束未经授权的远端会话，也不要把敏感资料明文上传到 Cloud。",
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
            "连接与认证",
            vec![link("connection-auth", "02", "首次连接与主机密钥")],
        ),
        group(
            "Agent 与终端",
            vec![link("agent-terminal", "03", "部署与持久化终端")],
        ),
        group("监控", vec![link("monitoring", "04", "快照、实时与历史")]),
        group(
            "文件与系统",
            vec![link("files-system", "05", "结构化文件和系统操作")],
        ),
        group(
            "端口映射",
            vec![link("port-forwarding", "06", "本地 SSH 转发")],
        ),
        group("AI", vec![link("ai", "07", "自带模型与工具确认")]),
        group(
            "Cloud 与安全",
            vec![link("cloud-security", "08", "可选账号与数据边界")],
        ),
        group(
            "故障排查",
            vec![link("troubleshooting", "09", "安全停止条件")],
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
                    "打开 v0.6.16 发布页，按平台矩阵选择七个正式安装/发布资产（不含 GitHub 自动源码归档）中的一个。不要从聊天附件、网盘转存或不明镜像安装。",
                    true,
                ),
                command(
                    "SHA256",
                    "逐字核对校验值",
                    "对下载文件计算 SHA256，并与该版本 Release notes 中同名资产的值逐字比较。若发布说明未提供该资产的 SHA256，或任一字符不一致，立即停止。",
                    "Windows: Get-FileHash .\\<asset> -Algorithm SHA256\nLinux:   sha256sum ./<asset>",
                    true,
                ),
                text(
                    "INSTALL",
                    "只安装与你平台匹配的变体",
                    "Windows 选择 NSIS、MSI 或便携包之一；Linux 在 deb 与 AppImage 中选择；Android 普通安装使用已签名 APK。macOS 与 iOS 没有下载。",
                    false,
                ),
            ],
        ),
        section(
            "connection-auth",
            "02 / CONNECTION & AUTH",
            "首次连接与主机密钥",
            "示例值只用于说明字段，真实连接必须由你确认服务器身份。",
            vec![
                command(
                    "EXAMPLE",
                    "添加一台示例主机",
                    "主机可填写 example.com 或 RFC 5737 示例地址 192.0.2.10，端口默认 22，再选择密码或私钥认证。真实凭据只交给本地客户端，不要写入网页、文档或上传到 agent。",
                    "Host: example.com\nAddress: 192.0.2.10\nPort: 22\nAuth: password or private key",
                    false,
                ),
                text(
                    "TRUST",
                    "显式确认首次主机密钥",
                    "首次连接时在独立可信渠道核对指纹，然后明确接受。以后若主机密钥变化，先查明服务器重装、DNS 或代理变化原因；禁止静默接受。",
                    true,
                ),
                text(
                    "FALLBACK",
                    "未安装 agent 仍可用普通 PTY",
                    "普通终端走原生 SSH PTY，端口映射也走 SSH 原生能力。持久化终端、结构化监控、文件、系统、应用与 AI 工具需要 agent。",
                    false,
                ),
            ],
        ),
        section(
            "agent-terminal",
            "03 / AGENT & TERMINAL",
            "部署 agent 并验证持久化终端",
            "客户端自动探测架构，只上传匹配的 agent 与静态 tmux 配对资源。",
            vec![
                text(
                    "DETECT",
                    "先只读探测，再上传匹配资源",
                    "已认证 SSH 先执行只读 uname -m。x86_64 与 aarch64 资源必须成对齐全；不支持的架构或缺少任一资源时，应在建目录、上传或停旧服务前失败。aarch64 构建与自动选择基础已验证，但因缺真实 ARM64 测试服务器，当前不宣称完整 ARM64 真机支持。",
                    true,
                ),
                text(
                    "SESSION",
                    "创建第一个持久化终端",
                    "选择持久化终端，让 agent 直驱 tmux 创建会话。普通 PTY 与持久化会话是两条明确路径；不要把未持久化的 shell 误认为可恢复会话。",
                    false,
                ),
                text(
                    "RECONNECT",
                    "断开客户端再回来验证",
                    "在会话中留下无敏感信息的标记，正常断开客户端但不要结束 tmux；重新连接后确认 capture-pane 恢复同一上下文。只有真实断线重连成功才算验证完成。",
                    false,
                ),
            ],
        ),
        section(
            "monitoring",
            "04 / MONITORING",
            "从快照进入实时与历史",
            "后台每轮对同一主机只取一个结构化 MetricsSnapshot，详情页实时流另行按需建立。",
            vec![
                text(
                    "SNAPSHOT",
                    "先看主机概览",
                    "确认 CPU、内存、磁盘、负载与 agent 状态能形成同一时间点快照。跨主机调度和单主机短请求保护是两层独立限制。",
                    false,
                ),
                text(
                    "HISTORY",
                    "再检查实时与历史",
                    "打开详情时使用按需长流；历史数据由 agent 本地 redb 保存。不要把重复短轮询伪装成实时订阅。",
                    false,
                ),
            ],
        ),
        section(
            "files-system",
            "05 / FILES & SYSTEM",
            "在当前主机上下文做结构化操作",
            "文件、进程、防火墙、systemd 与应用操作应由 agent 返回结构化结果，而不是由界面拼 shell。",
            vec![
                text(
                    "FILES",
                    "先从只读浏览开始",
                    "确认路径、权限和目标主机后，再执行编辑、上传或下载。高风险覆盖必须保持确认与失败可见，断点续传按传输会话规则治理。",
                    true,
                ),
                text(
                    "SYSTEM",
                    "高风险动作逐项确认",
                    "停止进程、修改防火墙或服务状态前核对对象归属。不得结束陌生进程、用户 SSH 会话或未经授权的 tmux 工作。",
                    true,
                ),
            ],
        ),
        section(
            "port-forwarding",
            "06 / PORT FORWARDING",
            "使用原生 SSH 本地转发",
            "端口映射是保留的纯 SSH 例外，不依赖 agent。",
            vec![command(
                "LOOPBACK",
                "默认仅绑定本机回环",
                "把服务器可访问的目标端口映射到本机 127.0.0.1。只有明确理解暴露面时才调整监听地址。",
                "127.0.0.1:<local-port> -> <remote-host>:<remote-port>",
                true,
            )],
        ),
        section(
            "ai",
            "07 / AI ASSISTANT",
            "使用自己的模型并保留工具确认",
            "AI 是对结构化 agent 能力的编排层，不应绕过权限、确认和审计。",
            vec![
                text(
                    "MODEL",
                    "模型配置留在本地",
                    "选择兼容的自带模型提供方；密钥不得出现在日志、文档或 agent 上。先用不调用工具的问题验证模型连接。",
                    true,
                ),
                text(
                    "TOOLS",
                    "工具调用先看目标与参数",
                    "涉及主机读取或修改时，核对目标主机、工具名和参数，再明确确认。不要允许 AI 通过自由 shell 绕过结构化边界。",
                    true,
                ),
            ],
        ),
        section(
            "cloud-security",
            "08 / CLOUD & SECURITY",
            "Cloud 账号可选，数据面仍走 SSH",
            "当前本地实现尚未部署、也未连接线上服务；不要把页面存在误述为云端能力已经交付。",
            vec![
                text(
                    "OPTIONAL",
                    "无需 Cloud 账号也可管理本地主机",
                    "SSH 连接、普通终端和本地工作流不以 Cloud 登录为前提。Cloud 未来用于账号、设备、同步、模型配置、保险库信封、版本与下载控制面。",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud 不代理 SSH 数据面",
                    "私钥、密码和明文敏感资料不上云。只有专项方案定义的客户端本地加密 vault 信封可存储；当前本地 Cloud 实现未部署、未接入生产。",
                    true,
                ),
            ],
        ),
        section(
            "troubleshooting",
            "09 / TROUBLESHOOTING",
            "遇到异常时安全停止",
            "先保住身份与远端工作，再定位网络、架构、资源和权限。",
            vec![
                text(
                    "HOST KEY",
                    "主机密钥变化",
                    "停止连接，使用可信渠道核对新指纹与变更原因。不要删除 known_hosts 记录来跳过确认。",
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
                    "确认使用的是持久化终端而非普通 PTY，并检查 agent 与 tmux 归属。未经授权不要 kill 远端会话或清理未知 socket。",
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
