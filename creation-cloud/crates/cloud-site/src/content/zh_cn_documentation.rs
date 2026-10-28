//! 提供简体中文文档入口、实操指南与安全参考内容。

use crate::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationScreenshot, DocumentationSection, PageContent, PageId,
};

use super::zh_cn::{action, page};

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Documentation,
        "C-SSH 使用文档｜连接与运维实操指南",
        "C-SSH 0.9.7使用文档：经典／工作台、Agent接入、持久终端、监控并发、服务器浏览器、AI进度与用量、手动云同步和文件管理。",
        "DOCS / TASK GUIDES",
        "从第一台主机开始使用 C-SSH",
        "按C-SSH 0.9.7的目录完成一个任务；可以先预览CSV导入、跳过轻量引导，再按需使用终端、监控与AI。",
    )
    .with_actions(vec![
        action("添加第一台主机", "#add-host", "button button-primary"),
        action("查看安全边界", "/security", "button button-secondary"),
    ])
    .with_documentation_page(documentation())
}

fn documentation() -> DocumentationContent {
    DocumentationContent {
        index_label: "文档目录",
        mobile_index_label: "展开文档目录",
        search_label: "筛选本页标题",
        search_placeholder: "例如：主机、终端、监控",
        search_help: "只筛选当前页面已经加载的标题，不搜索正文，也不会跳转到站外。",
        search_empty: "当前页没有匹配的标题。",
        status: DocumentationNotice::new(
            "开始前",
            "先确认主机身份与连接模式",
            "Agent 模式提供完整联动能力；SSH 模式提供原生连接能力。首次连接必须核对主机密钥，密钥变化时停止并显式确认。",
        ),
        groups: groups(),
        tutorials: super::zh_cn_tutorials::content(),
        sections: sections(),
        screenshot: DocumentationScreenshot {
            code: "PRODUCT VIEW / REDACTED DEMO",
            title: "普通 PTY 与持久化终端是两条不同路径",
            lead: "图中是直连普通 PTY；切换到持久化终端后，才由客户端、agent 与 tmux 协作提供可重连会话。",
            src: "/static/img/product-terminal.png",
            alt: "C-SSH 脱敏演示终端，显示示例服务器和普通 SSH PTY",
            caption: "脱敏演示图：地址使用 RFC 5737 示例值；只说明界面路径，不作为持久化会话或 no-mock 证据。",
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
            vec![link("getting-started", "01", "使用前准备")],
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
                link("ai-usage-and-tasks", "11", "AI并行任务与用量"),
                link("server-browser", "12", "独立服务器浏览器"),
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

fn sections() -> Vec<DocumentationSection> {
    vec![
        section(
            "getting-started",
            "01 / QUICK START",
            "使用前准备",
            "打开已安装的客户端后，先确定连接模式、主机身份与数据边界。",
            vec![
                text(
                    "0.9.7",
                    "AI、服务器浏览器与同步改进",
                    "AI任务并行与实时进度、上下文和缓存统计、独立服务器浏览器、监控队列与选择性压缩、手动云同步及Android终端和历史修复。完整变化见更新日志。",
                    false,
                ),
                text(
                    "MODE",
                    "选择 Agent 或 SSH 模式",
                    "默认使用 Agent 模式获得持久终端、监控和结构化运维；只需原生连接或目标机不适合 agent 时选择 SSH 模式。",
                    true,
                ),
                text(
                    "HOST KEY",
                    "核对主机身份",
                    "首次连接先核对主机密钥指纹；已知主机的密钥发生变化时必须停止，不要静默接受。",
                    true,
                ),
                text(
                    "DATA",
                    "确认 Cloud 数据边界",
                    "SSH 数据面始终由客户端直连服务器；Creation Cloud 只保存账号、设备和允许同步的数据，不代理终端或远程命令。",
                    false,
                ),
            ],
        ),
        section(
            "ai-usage-and-tasks",
            "11 / AI TASKS & USAGE",
            "查看AI进度、并发与用量",
            "在设置的AI模型执行设置中调整独立任务并发；需要时展开聊天中的执行详情。",
            vec![
                text(
                    "PROGRESS",
                    "及时看见结果",
                    "主画面保留简洁摘要，子任务进度和已有结果持续更新；插话、暂停、停止及接续沿当前会话处理。",
                    false,
                ),
                text(
                    "USAGE",
                    "累计与上下文分开",
                    "累计使用Token包含历次请求及子任务；上下文是主助手单次输入，不是累计总量。缓存缺失显示未报告，刷新期间保留上次确认值。",
                    false,
                ),
                text(
                    "MEMORY",
                    "保留原始记录",
                    "上下文按模型实际窗口处理，长会话采用有界读取；AI对话与原始记忆保存在本机，不参与Cloud配置同步。",
                    false,
                ),
            ],
        ),
        section(
            "server-browser",
            "12 / SERVER BROWSER",
            "使用独立服务器浏览器环境",
            "从桌面主机的服务器浏览器入口进入列表；主机需有可用Agent，本机需安装Google Chrome。",
            vec![
                text(
                    "ENVIRONMENT",
                    "一个环境一套资料",
                    "同一服务器可新增多个环境，各自保存浏览数据、固定种子和设置。打开前检查Chrome与Agent能力；关闭或重开不重置已有身份。",
                    false,
                ),
                text(
                    "SETTINGS",
                    "系统、语言与权限",
                    "在环境设置中选择系统、时区、语言及权限。定位、麦克风和摄像头按设置正常询问；网站仍可能要求验证。",
                    false,
                ),
                text(
                    "MOBILE",
                    "当前平台边界",
                    "Windows、Mac和Linux提供此入口；Android入口隐藏且验证暂停，iOS继续暂停。",
                    true,
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
            "Creation Cloud 生产控制面已经部署；Cloud 账号仍不是本地 SSH 工作流的前提。",
            vec![
                text(
                    "OPTIONAL",
                    "无需 Cloud 账号也可管理本地主机",
                    "SSH连接、普通终端和本地工作流不以Cloud登录为前提。Cloud只承载账号、设备、模型目录与用户主动发起的主机、AI账户及代理配置密文同步。",
                    false,
                ),
                text(
                    "BOUNDARY",
                    "Cloud 不代理 SSH 数据面",
                    "主机、AI账户及代理配置只以客户端加密的不透明密文手动同步；数据保护密码与密钥不上传，AI对话和长期记忆不参与Cloud同步。",
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
