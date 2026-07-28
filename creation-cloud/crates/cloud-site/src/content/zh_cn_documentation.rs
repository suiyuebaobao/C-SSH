//! 提供简体中文文档入口、实操指南与安全参考内容。

use crate::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationScreenshot, DocumentationSection, PageContent, PageId,
};

use super::zh_cn::{action, page};

pub(super) fn page_content() -> PageContent {
    page(
        PageId::Documentation,
        "Creation-SSH 使用文档｜连接与运维实操指南",
        "从添加主机到 agent 部署、持久终端、监控、文件与 AI 的 Creation-SSH 使用文档。",
        "DOCS / TASK GUIDES",
        "从第一台主机开始使用 Creation-SSH",
        "文档和实操指南已经合并。按目录完成一个真实任务，并用每章的预期结果判断链路是否可用。",
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
            alt: "Creation-SSH 脱敏演示终端，显示示例服务器和普通 SSH PTY",
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
                    "SSH 连接、普通终端和本地工作流不以 Cloud 登录为前提。Cloud 仅承载账号、设备、同步、模型与保险库信封等控制面数据。",
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
