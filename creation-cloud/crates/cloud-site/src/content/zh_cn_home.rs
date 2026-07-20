//! 提供简体中文工业制图首页的完整产品信息结构。

use crate::{
    HomeFaqItem, HomeItem, HomeLayout, HomePageContent, HomePlatform, HomeQrLabels, HomeQrWidget,
    HomeSection, HomeTone, PageContent, PageId,
};

use super::zh_cn::{action, page};

pub(super) fn page_content() -> PageContent {
    let platforms = platforms();
    let sections = sections(&platforms);
    let home_page = HomePageContent {
        status_note: "本地实现 · 未部署 · 未发布",
        platform_label: "I/O MATRIX / PLATFORM",
        platform_note: "5 SLOTS",
        platforms,
        sections,
        faq_code: "08 / FAQ",
        faq_heading: "把决定是否使用前的问题说清楚",
        faq_lead: "只保留高价值决策问题，完整解释继续进入常见问题页。",
        faqs: faqs(),
        final_code: "NEXT STEP / START HERE",
        final_heading: "先选平台，再从第一条教程开始",
        final_lead: "所有能力、平台状态与下载信息都以真实实现、部署和发布记录为准。",
        qr_widget: HomeQrWidget::pending(HomeQrLabels {
            code: "MOBILE ACCESS / QR",
            title: "扫码入口",
            pending: "二维码待后台上传",
            ready: "二维码已发布",
            note: "点击卡片切换大小",
            image_alt: "Creation-SSH 二维码",
            open: "放大查看二维码",
            close: "收起二维码",
        }),
    };

    page(
        PageId::Home,
        "Creation-SSH｜SSH 客户端与服务器运维工具",
        "Creation-SSH 是原生 SSH 客户端与服务器运维工具，通过 SSH 隧道和常驻 agent 提供持久终端、监控、文件与系统能力。",
        "CLIENT × RESIDENT AGENT",
        "让 SSH 工作现场，始终在线",
        "原生客户端与服务器常驻 agent 协作：持久终端、常驻监控、文件与系统能力，都回到同一台主机上下文。",
    )
    .with_actions(vec![
        action(
            "阅读入门文档",
            "/docs/getting-started",
            "button button-primary",
        ),
        action("查看更新日志", "/changelog", "button button-secondary"),
        action("按教程开始", "/tutorials", "text-link"),
    ])
    .with_home_page(home_page)
}

fn platforms() -> Vec<HomePlatform> {
    vec![
        HomePlatform::current(
            "W",
            "Windows",
            "桌面客户端",
            "完整桌面运维体验",
            "独立原生客户端",
            "以下载页真实记录为准",
        ),
        HomePlatform::current(
            "L",
            "Linux",
            "独立客户端",
            "桌面运维与独立构建",
            "独立原生客户端",
            "以下载页真实记录为准",
        ),
        HomePlatform::current(
            "A",
            "Android",
            "移动伴侣",
            "移动查看与轻量操作",
            "独立移动客户端",
            "以下载页真实记录为准",
        ),
        HomePlatform::planned(
            "m",
            "macOS",
            "独立桌面客户端",
            "未来桌面客户端",
            "预留独立平台边界",
            "规划中 · 未开放下载",
        ),
        HomePlatform::planned(
            "i",
            "iOS",
            "独立移动伴侣",
            "未来移动伴侣",
            "预留独立平台边界",
            "规划中 · 未开放下载",
        ),
    ]
}

fn sections(platforms: &[HomePlatform]) -> Vec<HomeSection> {
    vec![
        section(
            "workflow",
            HomeLayout::Workflow,
            "客户端与 agent，分工必须一眼看懂",
            "客户端负责认证与发起，SSH 是唯一入口；常驻 agent 只通过服务器本机 socket 提供结构化能力。",
            vec![
                item(
                    "NODE 01",
                    "原生客户端",
                    "主机、凭据、信任与操作入口留在本地。",
                    "CLIENT",
                ),
                item(
                    "LINK 02",
                    "SSH 隧道",
                    "复用 SSH 认证与加密，不增加公网端口。",
                    "PURE SSH",
                ),
                item(
                    "CORE 03",
                    "常驻 agent",
                    "仅监听服务器本机 socket，持续提供结构化能力。",
                    "AGENT",
                ),
                item(
                    "STATE 04",
                    "tmux / redb",
                    "终端现场与监控历史在服务器侧持续存在。",
                    "SERVER STATE",
                ),
                item(
                    "TOOLS 05",
                    "文件与系统",
                    "可靠传输和系统操作经明确协议执行。",
                    "STRUCTURED",
                ),
                item(
                    "PURE SSH",
                    "未安装 agent 仍可使用",
                    "普通终端、端口映射与访问授权保留原生 SSH 路径。",
                    "兜底边界",
                ),
                item(
                    "RESIDENT AGENT",
                    "客户端 × agent 联动",
                    "持久终端、监控历史、文件、AI、系统与应用中心走结构化协议。",
                    "主打路径",
                ),
            ],
        ),
        section(
            "capabilities",
            HomeLayout::Capabilities,
            "一套工作台，九个边界清楚的模块",
            "每个模块都说明运行依赖与实际结果；完整架构、边界和界面说明继续进入产品文档。",
            vec![
                item(
                    "MIXED",
                    "主机管理",
                    "连接、认证、主机密钥与真实在线结果。",
                    "连接入口",
                ),
                item(
                    "MIXED",
                    "终端",
                    "持久 tmux 与普通 SSH PTY 按场景选择。",
                    "现场恢复",
                ),
                item(
                    "AGENT",
                    "监控",
                    "快照、实时订阅、历史与 Top 进程。",
                    "结构化状态",
                ),
                item(
                    "AGENT",
                    "文件",
                    "浏览、搜索、编辑和带校验的可靠传输。",
                    "同一主机上下文",
                ),
                item(
                    "PURE SSH",
                    "端口映射",
                    "将服务器可达端口安全映射到客户端本机。",
                    "SSH 原生",
                ),
                item(
                    "AGENT",
                    "AI 助手",
                    "在权限、确认与审计门内调用结构化工具。",
                    "自有模型",
                ),
                item(
                    "AGENT",
                    "系统管理",
                    "读取系统事实，受控管理进程与防火墙。",
                    "显式操作",
                ),
                item(
                    "AGENT",
                    "应用中心",
                    "聚焦 Docker、常用应用与 systemd 服务。",
                    "能力模块",
                ),
                item(
                    "PURE SSH",
                    "访问授权",
                    "生成独立访问钥，并按稳定标记精确吊销。",
                    "最小授权",
                ),
            ],
        ),
        section(
            "first-run",
            HomeLayout::Steps,
            "从第一台主机，到完整工作流",
            "教程不是散乱的帮助文章，而是从首次连接到日常运维的可执行路径。",
            vec![
                item(
                    "STEP 01",
                    "添加主机",
                    "完成 SSH 认证与主机密钥信任。",
                    "连接验证",
                ),
                item(
                    "STEP 02",
                    "部署 agent",
                    "探测服务器架构并选择匹配的 agent 与 tmux。",
                    "配对部署",
                ),
                item(
                    "STEP 03",
                    "持久终端",
                    "主动断开，再验证任务与屏幕现场恢复。",
                    "tmux",
                ),
                item(
                    "STEP 04",
                    "监控历史",
                    "查看实时详情、历史范围与进程。",
                    "redb",
                ),
                item(
                    "STEP 05",
                    "文件传输",
                    "上传或下载，并核对完整性结果。",
                    "分块校验",
                ),
                item(
                    "STEP 06",
                    "AI 助手",
                    "使用自己的模型，从只读诊断开始。",
                    "权限门",
                ),
            ],
        ),
        section(
            "platforms",
            HomeLayout::Platforms,
            "五个平台，各自独立，不合并占位",
            "Windows、Linux、Android 的公开状态以真实发布记录为准；macOS 与 iOS 始终保留独立产品位。",
            platform_items(platforms),
        ),
        section(
            "security",
            HomeLayout::Security,
            "SSH 数据面直接，云端控制面克制",
            "两条链路必须分开解释：Creation Cloud 不进入客户端与用户服务器之间的 SSH 数据面。",
            vec![
                item(
                    "SSH DATA PLANE",
                    "客户端直连用户服务器",
                    "agent 仅监听本机 socket；主机密钥变化显式确认；不私自结束 SSH、tmux 或用户进程。",
                    "Cloud 不在链路中",
                ),
                item(
                    "CLOUD CONTROL PLANE",
                    "账号、设备与可选同步",
                    "只同步非敏感白名单；保险库加解密留在可信客户端，服务端只保存版本化密文信封。",
                    "敏感资料不明文上云",
                ),
            ],
        ),
        section(
            "downloads",
            HomeLayout::Downloads,
            "下载前，先确认来源与校验值",
            "首页只读取最近真实版本；发布数据未接入时保持明确空状态，不写死版本号或占位链接。",
            vec![
                item(
                    "WINDOWS",
                    "等待发布数据",
                    "接入后显示架构、来源、大小与发布时间。",
                    "SHA256：—",
                ),
                item(
                    "LINUX",
                    "等待发布数据",
                    "AppImage 与 deb 必须来自真实本地构建记录。",
                    "SHA256：—",
                ),
                item(
                    "ANDROID",
                    "等待发布数据",
                    "正式 arm64 资产与测试用 x86_64 包保持分离。",
                    "SHA256：—",
                ),
                item(
                    "macOS",
                    "规划中",
                    "不提供下载动作或已支持暗示。",
                    "SHA256：—",
                ),
                item("iOS", "规划中", "不提供下载动作或已支持暗示。", "SHA256：—"),
            ],
        ),
        section(
            "cloud",
            HomeLayout::Cloud,
            "账号服务于设备与同步，不介入 SSH",
            "Creation Cloud 控制面当前为本地实现阶段：未部署、未发布、未接入客户端；页面只说明控制面的正向价值与真实状态。",
            vec![
                item(
                    "ACCOUNT",
                    "资料与安全",
                    "账号状态、安全设置与密码边界。",
                    "账号密码 ≠ 保险库密码",
                ),
                item(
                    "DEVICE",
                    "设备",
                    "登记、重命名与撤销可信设备。",
                    "不保存 SSH 主机资料",
                ),
                item(
                    "SYNC",
                    "同步",
                    "revision、冲突与非敏感白名单结果。",
                    "默认拒绝未知字段",
                ),
                item(
                    "MODEL",
                    "模型",
                    "同步无凭据的模型元数据、默认项与排序。",
                    "API Key 只引用密文",
                ),
                item(
                    "VAULT",
                    "保险库",
                    "只显示密文版本与设备包装状态。",
                    "服务端不可解密",
                ),
                item(
                    "RELEASE",
                    "下载",
                    "展示兼容版本、来源和下载历史。",
                    "真实记录",
                ),
            ],
        ),
    ]
}

fn faqs() -> Vec<HomeFaqItem> {
    vec![
        HomeFaqItem::new(
            "Creation Cloud 会代理 SSH 吗？",
            "不会。SSH 数据面保持客户端直连用户服务器，Cloud 只承担账号、设备与可选同步等控制面能力。",
        ),
        HomeFaqItem::new(
            "没有安装 agent 还能使用吗？",
            "可以使用普通 SSH 终端、端口映射与访问授权；持久会话、监控和结构化管理能力依赖 agent。",
        ),
        HomeFaqItem::new(
            "主机地址和私钥会上云吗？",
            "不会明文上云。主机资料、密码、私钥、known_hosts、终端内容和命令历史均不在允许同步的白名单内。",
        ),
        HomeFaqItem::new(
            "保险库密码与账号密码相同吗？",
            "不同。账号密码只负责登录；保险库密码只在可信客户端派生加密密钥，不上传服务端。",
        ),
        HomeFaqItem::new(
            "如何核对下载文件没有被替换？",
            "正式下载条目会展示平台、架构、来源、文件大小与 SHA256，安装前可逐项核对。",
        ),
        HomeFaqItem::new(
            "Android 是桌面端的完整复制吗？",
            "不是。Android 是移动伴侣，优先覆盖查看、轻量操作和与桌面工作流衔接的场景。",
        ),
    ]
}

fn section(
    anchor: &'static str,
    layout: HomeLayout,
    title: &'static str,
    lead: &'static str,
    items: Vec<HomeItem>,
) -> HomeSection {
    let (code, side_label) = match layout {
        HomeLayout::Workflow => ("01 / HOW IT WORKS", "SYSTEM FLOW / DATA PATH"),
        HomeLayout::Capabilities => ("02 / CAPABILITIES", "FUNCTION MODULE / 09 UNITS"),
        HomeLayout::Steps => ("03 / FIRST RUN", "OPERATION SEQUENCE / 06 STEPS"),
        HomeLayout::Platforms => ("04 / PLATFORMS", "PLATFORM MATRIX / 05 SLOTS"),
        HomeLayout::Security => ("05 / SECURITY BOUNDARY", "SEPARATED PLANES"),
        HomeLayout::Downloads => ("06 / DOWNLOADS", "SOURCE AND CHECKSUM"),
        HomeLayout::Cloud => ("07 / CREATION CLOUD", "CONTROL SURFACE"),
    };
    HomeSection::new(anchor, code, side_label, layout, title, lead, items)
}

fn item(
    badge: &'static str,
    title: &'static str,
    body: &'static str,
    meta: &'static str,
) -> HomeItem {
    let tone = match badge {
        "CORE 03" | "SSH DATA PLANE" => HomeTone::Dark,
        "PLANNED" | "macOS" | "iOS" => HomeTone::Planned,
        "AGENT" | "CLOUD CONTROL PLANE" | "RESIDENT AGENT" => HomeTone::Accent,
        _ => HomeTone::Default,
    };
    HomeItem::new(badge, title, body, meta, tone)
}

fn platform_items(platforms: &[HomePlatform]) -> Vec<HomeItem> {
    platforms
        .iter()
        .map(|platform| {
            HomeItem::new(
                platform.state,
                platform.name,
                platform.position,
                platform.shell,
                if platform.planned {
                    HomeTone::Planned
                } else {
                    HomeTone::Default
                },
            )
        })
        .collect()
}
