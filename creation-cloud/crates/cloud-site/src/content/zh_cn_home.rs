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
        status_strip_label: "OFFICIAL PRODUCT SURFACE / INDUSTRIAL SYSTEM".into(),
        status_note: "0.9.2 · Windows / M系列Mac / Android / Linux · 必须手动下载安装".into(),
        hero_blueprint_label: "BRAND GEOMETRY / UNIT 01".into(),
        platform_label: "I/O MATRIX / PLATFORM".into(),
        platform_note: "4 RELEASED / iOS NOT PUBLISHED".into(),
        platforms,
        sections,
        faq_side_label: "FAQ / DECISION QUESTIONS".into(),
        faq_item_prefix: "FAQ".into(),
        faq_code: "08 / FAQ".into(),
        faq_heading: "把决定是否使用前的问题说清楚".into(),
        faq_lead: "只保留高价值决策问题，完整解释继续进入常见问题页。".into(),
        faqs: faqs(),
        seo_code: "SEARCH TOPICS / CONTENT MAP".into(),
        seo_heading: "相关 SSH 主题".into(),
        seo_topics_label: "已发布的 SEO 主题词".into(),
        final_code: "NEXT STEP / START HERE".into(),
        final_heading: "下载前，先保全已有数据".into(),
        final_lead: "本次仅通过公告告知，不推送升级提醒，不强制更新。请手动下载安装，并先阅读安装与数据保全说明；Mac自动更新延期。".into(),
        qr_placeholder_code: "QR".into(),
        qr_placeholder_waiting: "WAITING".into(),
        media_slot: None,
        qr_widget: HomeQrWidget::pending(HomeQrLabels {
            code: "QQ COMMUNITY / QR",
            title: "加入 QQ 交流群",
            pending: "QQ 群二维码待上传",
            ready: "QQ 群二维码已发布",
            note: "扫码加入 C-SSH 用户群 · 点击可放大",
            image_alt: "加入 C-SSH QQ 交流群的二维码",
            open: "放大 QQ 群二维码",
            close: "收起 QQ 群二维码",
        }),
    };

    page(
        PageId::Home,
        "C-SSH 0.9.2 | Windows · Mac · Android · Linux",
        "C-SSH 0.9.2 支持Windows x64／ARM64、M系列Mac、Android和Linux，提供SSH、RDP、持久终端、监控、文件、三作用域AI及可选自建云；本次仅发布公告，手动下载安装，不推送升级提醒或强制更新。",
        "C-SSH 0.9.2 / MANUAL DOWNLOAD",
        "C-SSH 0.9.2，把服务器运维放进同一工作台",
        "Windows、M系列Mac与Android：主机、持久终端、远程桌面、监控、文件和AI助手统一使用。本次仅通过公告告知，请手动下载安装；不推送升级提醒，不强制更新。iOS本次不提供下载。",
    )
    .with_actions(vec![
        action(
            "手动下载 0.9.2",
            "/downloads",
            "button button-primary",
        ),
        action("完整功能介绍", "https://github.com/suiyuebaobao/C-SSH/blob/main/FEATURES.md", "button button-secondary"),
        action("三端完整图集", "https://github.com/suiyuebaobao/C-SSH/blob/main/screenshots/README.md", "text-link"),
    ])
    .with_home_page(home_page)
}

fn platforms() -> Vec<HomePlatform> {
    vec![
        HomePlatform::current(
            "W",
            "Windows",
            "x64 / ARM64",
            "完整桌面运维体验",
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
        HomePlatform::current(
            "m",
            "macOS",
            "Apple Silicon / arm64",
            "M系列arm64桌面客户端",
            "ad-hoc签名 · 未公证",
            "DMG · 手动下载",
        ),
        HomePlatform::current(
            "L",
            "Linux",
            "x86_64 / ARM64",
            "完整桌面客户端",
            "Ubuntu 24.04 / X11已验证",
            "deb / AppImage · 手动下载",
        ),
        HomePlatform::planned(
            "i",
            "iOS",
            "iPhone客户端开发中",
            "本次不提供下载",
            "不上传IPA或TestFlight",
            "本次未发布",
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
                    "三个客户端",
                    "Windows x64、M系列Mac arm64和Android arm64共享核心能力，按桌面和手机布局使用。",
                    "0.9.2 · 手动安装",
                ),
                item(
                    "LINK 02",
                    "SSH与RDP连接",
                    "在同一地址框直接输入IPv4或域名，自动识别格式；端口独立填写，已有IPv6主机保持兼容。",
                    "PURE SSH",
                ),
                item(
                    "CORE 03",
                    "Agent与普通SSH",
                    "Agent模式提供结构化管理与常驻能力；普通SSH模式使用PTY、SFTP等现有服务。",
                    "AGENT",
                ),
                item(
                    "STATE 04",
                    "持久终端",
                    "Linux使用tmux；Windows使用配套Agent的Terminal2／psmux。关闭客户端后可重新连接原会话。",
                    "不保证所有故障均可恢复",
                ),
                item(
                    "TOOLS 05",
                    "文件与系统",
                    "文件浏览、传输、编辑和校验；监控、进程、防火墙及服务能力按目标主机支持情况开放。",
                    "Windows与Linux能力独立识别",
                ),
                item(
                    "PURE SSH",
                    "内置Windows安装包",
                    "添加Windows管理主机时，可从客户端保存内置安装ZIP；Android还可系统分享。传到服务器后解压并手动运行Setup。",
                    "安装包离线可取 · 不自动安装",
                ),
                item(
                    "RESIDENT AGENT",
                    "本地优先",
                    "不登录Cloud也能使用本地运维；可选账号用于设备和用户主动发起的端到端加密同步。",
                    "云不转发SSH数据",
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
                    "主机与项目",
                    "新增、编辑、搜索、收藏和分组；支持IPv4与域名、独立端口、密码或私钥，以及直连和代理线路。",
                    "主机 / 项目",
                ),
                item(
                    "MIXED",
                    "终端与远程桌面",
                    "持久终端和普通SSH终端；桌面端多标签与独立RDP窗口，按平台提供键盘、剪贴板和文件交互。",
                    "SSH / RDP",
                ),
                item(
                    "AGENT",
                    "监控与历史",
                    "CPU、内存、磁盘、负载、网络及磁盘IO；实时曲线、历史区间、Top进程和采集设置。",
                    "实际可用字段按服务端返回",
                ),
                item(
                    "AGENT",
                    "文件管理",
                    "目录浏览、上传、下载、创建、重命名、删除、文本编辑和摘要校验；手机配合系统文件选择器。",
                    "操作前确认目标",
                ),
                item(
                    "PURE SSH",
                    "端口映射与群发",
                    "Windows和Mac提供SSH端口映射、常用命令库、批量执行和逐机结果；Android不提供这两类页面。",
                    "桌面功能",
                ),
                item(
                    "AGENT",
                    "AI 助手",
                    "全局可访问本机所有作用域历史，项目限定本项目，主机限定当前主机；支持模型账户、绑定、权限模式和操作确认。",
                    "对话与记忆保存在本机",
                ),
                item(
                    "AGENT",
                    "系统管理",
                    "系统信息、进程与防火墙；Windows Native管理和Linux Agent按握手能力提供相应入口。",
                    "不把未知能力当成已支持",
                ),
                item(
                    "AGENT",
                    "应用中心",
                    "桌面端集中查看和管理受支持的Docker容器、镜像、systemd及Windows服务。",
                    "能力由目标服务器决定",
                ),
                item(
                    "PURE SSH",
                    "授权与诊断",
                    "桌面访问授权管理；诊断日志默认关闭，按需打开、查询与导出支持材料。",
                    "不同平台入口按实际界面",
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
                    "手动下载安装",
                    "本次0.9.2只通过公告告知，请从官网下载页或官方GitHub手动下载匹配平台的安装包。",
                    "核对版本和SHA256",
                ),
                item(
                    "STEP 02",
                    "保护已有数据",
                    "Windows安装与Mac替换应用时保留原数据；不同签名的Android旧版不能直接覆盖，卸载前必须确认数据已保全。",
                    "AI历史不参与云同步",
                ),
                item(
                    "STEP 03",
                    "持久终端",
                    "直接填写IPv4或域名、独立端口与凭据；根据目标选择Agent、普通SSH或Windows访问方式。",
                    "先确认主机身份",
                ),
                item(
                    "STEP 04",
                    "进入终端与监控",
                    "按需打开持久会话、普通终端和实时监控，再查看采集历史。",
                    "原有会话与数据保持",
                ),
                item(
                    "STEP 05",
                    "文件与项目协作",
                    "管理远端文件并按项目组织主机；桌面端可复用命令库和端口映射。",
                    "删除本地主机不卸载远端Agent",
                ),
                item(
                    "STEP 06",
                    "AI 助手",
                    "配置自有模型账户和权限；Cloud默认官方，也可在设置中选择兼容的自建Creation Cloud。",
                    "同步由用户主动发起",
                ),
            ],
        ),
        section(
            "platforms",
            HomeLayout::Platforms,
            "Windows、Mac、Android与Linux",
            "0.9.2发布Windows x64／ARM64、M系列Mac、Android和Linux；必须手动下载安装。Linux提供x86_64／ARM64两种架构；iOS本次不提供下载。",
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
                    "直连服务器并确认身份",
                    "SSH主机密钥和RDP证书的信任门禁保持；代理失败不会悄悄改成直连。删除本地主机不连接或清理远端。",
                    "控制面与数据面分离",
                ),
                item(
                    "CLOUD CONTROL PLANE",
                    "本地密钥与端到端同步",
                    "本地凭据由系统设备密钥保护，Cloud同步由账号数据保护密码及独立密钥加密。AI对话和记忆不上传Cloud；选定上下文会发送给所选模型服务商。",
                    "不登录也可使用本地功能",
                ),
            ],
        ),
        section(
            "cloud",
            HomeLayout::Cloud,
            "账号服务于设备与同步，不介入 SSH",
            "Creation Cloud 控制面已部署到正式服务器：账号、设备、同步、模型、保险库密文、版本、下载与后台已经提供；客户端正式接入另行发布。",
            vec![
                item(
                    "ACCOUNT",
                    "可选账号",
                    "注册、登录、设备管理和退出；登录不等于解密，也不会自动开始同步。",
                    "官方Cloud默认可用",
                ),
                item(
                    "DEVICE",
                    "官方或自建云",
                    "在设置中填写自己的HTTPS云地址，使用同一套Creation Cloud服务端。切换时隔离登录及同步状态。",
                    "第三方自行部署与维护",
                ),
                item(
                    "SYNC",
                    "手动加密同步",
                    "按需同步主机、代理线路和模型账户配置；密码、私钥、RDP凭据及模型Key只进入客户端加密后的密文。",
                    "不包含AI聊天与长期记忆",
                ),
                item(
                    "MODEL",
                    "模型账户与绑定",
                    "管理模型账户和自定义端点、选择模型绑定与上下文设置，按作用域和权限使用AI工具。",
                    "模型调用使用自己的服务账户",
                ),
                item(
                    "VAULT",
                    "数据保护",
                    "数据保护密码只用于Cloud数据，系统设备密钥保护本地数据；切换云不会自动搬运本地主机与聊天。",
                    "先确认目标云再同步",
                ),
                item(
                    "RELEASE",
                    "手动下载与更新来源",
                    "0.9.2必须手动下载；使用自建云时，后续更新检查走所选云，由其管理员上传来自官网的版本与原签名。",
                    "仅公告告知 · 不推送升级提醒或强制更新",
                ),
            ],
        ),
    ]
}

fn faqs() -> Vec<HomeFaqItem> {
    vec![
        HomeFaqItem::new(
            "0.9.2能从旧版自动更新吗？",
            "本次仅通过公告告知，不推送升级提醒，不强制更新。请手动下载安装，并先阅读安装与数据保全说明；Mac自动更新延期。",
        ),
        HomeFaqItem::new(
            "这次有哪些平台？",
            "Windows x64安装包与便携版、仅M系列Mac的arm64 DMG、Android arm64 APK。iOS本次不上传；Linux客户端保持冻结，Linux服务器Agent继续提供。",
        ),
        HomeFaqItem::new(
            "Android旧数据怎么办？",
            "不同签名的旧版不能直接覆盖。卸载会清除本地数据；请先确认所需数据已有独立备份。Cloud同步不包含AI对话与记忆，无法确认数据保全时请保留旧版。",
        ),
        HomeFaqItem::new(
            "必须使用官方云吗？",
            "不必。不登录也能使用本地功能；可在设置中选择兼容的自建Creation Cloud。切换前处理待恢复的同步，切换后登录目标云。",
        ),
        HomeFaqItem::new(
            "AI三种作用域有什么区别？",
            "全局可访问本机所有作用域历史，项目只访问本项目，主机只访问对应主机。它不会凭空拥有另一设备未同步的聊天；聊天和记忆不参与Cloud同步。",
        ),
        HomeFaqItem::new(
            "Mac有哪些安装限制？",
            "只提供M系列arm64版本，最低系统目标为macOS13。DMG内应用采用ad-hoc签名且未公证，系统可能要求确认；Mac自动更新延期，本次请手动安装。",
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
        "PLANNED" | "iOS" => HomeTone::Planned,
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
                &platform.state,
                &platform.name,
                &platform.position,
                &platform.shell,
                if platform.planned {
                    HomeTone::Planned
                } else {
                    HomeTone::Default
                },
            )
        })
        .collect()
}
