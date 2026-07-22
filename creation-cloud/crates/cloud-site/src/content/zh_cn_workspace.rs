//! 提供简体中文用户中心与管理后台内容。

use crate::{ContentSection, Metric, NavigationItem, PageContent, PageId};

use super::zh_cn::{item, nav, page, section};

pub(super) fn console_overview() -> PageContent {
    console_page(
        PageId::Console,
        "用户中心｜Creation Cloud",
        "总览",
        "你的 Creation Cloud 控制面",
        "从设备状态、同步修订到保险库包装状态，都在明确边界内呈现。",
        vec![
            Metric::new("—", "已登记设备", "登录后显示"),
            Metric::new("—", "同步修订", "登录后显示"),
            Metric::new("—", "保险库条目", "仅显示密文元数据"),
        ],
        vec![section(
            "overview",
            "账号概览",
            "业务服务接入后按真实账号状态呈现，不使用模拟统计。",
            vec![
                item(
                    "设备",
                    "设备状态",
                    "查看已登记设备与最近活动。",
                    "等待账号会话",
                ),
                item(
                    "同步",
                    "同步状态",
                    "查看 revision、冲突与最近同步。",
                    "等待账号会话",
                ),
                item(
                    "保险库",
                    "密文状态",
                    "查看条目版本与设备包装状态，不显示明文。",
                    "等待账号会话",
                ),
            ],
        )],
    )
}

pub(super) fn profile() -> PageContent {
    console_page(
        PageId::Profile,
        "资料与安全｜Creation Cloud",
        "资料",
        "管理个人资料与账号安全",
        "资料来自当前账号；修改密码不会接触保险库主密钥。",
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn devices() -> PageContent {
    console_page(
        PageId::Devices,
        "设备｜Creation Cloud",
        "设备",
        "管理已登记设备",
        "设备记录只标识客户端，不保存 SSH 主机资料。",
        vec![Metric::new("—", "设备数量", "登录后显示")],
        vec![section(
            "devices",
            "设备列表",
            "登记、重命名与撤销动作由设备服务提供。",
            vec![item(
                "空状态",
                "尚未载入设备",
                "登录后显示属于当前账号的设备。",
                "无模拟数据",
            )],
        )],
    )
}

pub(super) fn sync() -> PageContent {
    console_page(
        PageId::Sync,
        "同步｜Creation Cloud",
        "同步",
        "查看同步修订与冲突",
        "只同步白名单内的非敏感偏好，未知字段默认拒绝。",
        vec![
            Metric::new("—", "当前 revision", "登录后显示"),
            Metric::new("—", "待处理冲突", "登录后显示"),
        ],
        vec![section(
            "sync-state",
            "最近同步",
            "展示命名空间、revision 和结果，不展示敏感正文。",
            vec![item(
                "空状态",
                "尚未载入同步状态",
                "连接账号后显示真实同步记录。",
                "无模拟数据",
            )],
        )],
    )
}

pub(super) fn models() -> PageContent {
    console_page(
        PageId::Models,
        "模型｜Creation Cloud",
        "模型",
        "同步模型配置元数据",
        "名称、provider、模型 ID 与默认参数可同步，API Key 只能作为保险库密文引用。",
        vec![Metric::new("—", "模型配置", "登录后显示")],
        vec![section(
            "model-list",
            "模型列表",
            "业务服务接入后支持默认项、排序与启停。",
            vec![item(
                "空状态",
                "尚未载入模型",
                "不会用示例密钥或虚构配置填充页面。",
                "无模拟数据",
            )],
        )],
    )
}

pub(super) fn vault() -> PageContent {
    console_page(
        PageId::Vault,
        "保险库｜Creation Cloud",
        "保险库",
        "只管理版本化密文",
        "加密与解密发生在可信客户端；服务端看不到保险库明文和密码。",
        vec![
            Metric::new("—", "密文条目", "登录后显示"),
            Metric::new("—", "包装设备", "登录后显示"),
        ],
        vec![section(
            "vault-state",
            "保险库状态",
            "仅展示条目数量、版本与设备包装状态。",
            vec![item(
                "零知识",
                "尚未载入保险库",
                "登录后显示当前账号的密文元数据。",
                "不显示明文",
            )],
        )],
    )
}

pub(super) fn downloads() -> PageContent {
    console_page(
        PageId::ConsoleDownloads,
        "下载记录｜Creation Cloud",
        "下载",
        "查看兼容版本与下载历史",
        "版本与校验值只来自已发布记录；账号历史只显示真实关联事件。",
        Vec::new(),
        Vec::new(),
    )
}

pub(super) fn admin() -> PageContent {
    admin_page(
        PageId::Admin,
        "管理后台｜Creation Cloud",
        "控制面总览",
        "真实状态，一眼可核对",
        "查看进程、数据库、受控目录以及用户、设备、发布和审计的实时统计。",
    )
}

pub(super) fn admin_users() -> PageContent {
    admin_page(
        PageId::AdminUsers,
        "用户管理｜Creation Cloud",
        "账号治理",
        "管理用户与权限边界",
        "按脱敏身份查询账号，调整状态与角色，并保护当前和最后一个有效管理员。",
    )
}

pub(super) fn admin_devices() -> PageContent {
    admin_page(
        PageId::AdminDevices,
        "设备管理｜Creation Cloud",
        "设备治理",
        "只管理客户端设备元数据",
        "查看平台、版本和撤销状态；后台不会展示或保存任何 SSH 主机资料。",
    )
}

pub(super) fn admin_releases() -> PageContent {
    admin_page(
        PageId::AdminReleases,
        "版本管理｜Creation Cloud",
        "发布控制",
        "让版本状态可验证地前进",
        "创建版本、维护中英文说明，并按草稿、校验、发布、撤销或隐藏状态迁移。",
    )
}

pub(super) fn admin_assets() -> PageContent {
    admin_page(
        PageId::AdminAssets,
        "资产管理｜Creation Cloud",
        "交付资产",
        "文件身份、来源和校验保持一致",
        "登记平台资产，完成隔离上传与 SHA256 核验，再管理本站和 HTTPS 外部来源。",
    )
}

pub(super) fn admin_site() -> PageContent {
    admin_page(
        PageId::AdminSite,
        "站点资源｜Creation Cloud",
        "站点资源",
        "管理首页二维码发布槽位",
        "上传受控位图、预览草稿并发布同源版本；没有发布内容时首页继续显示空状态。",
    )
}

pub(super) fn admin_audit() -> PageContent {
    admin_page(
        PageId::AdminAudit,
        "审计记录｜Creation Cloud",
        "安全审计",
        "每次管理动作都可追溯",
        "按时间查看服务端自动生成的操作者、动作、资源、结果和脱敏请求标识。",
    )
}

pub(super) fn admin_feedback() -> PageContent {
    admin_page(
        PageId::AdminFeedback,
        "问题反馈｜Creation Cloud",
        "反馈处理",
        "查看并推进官网反馈",
        "列表只展示最小摘要；管理员显式打开详情后才能读取纯文本内容并更新处理状态。",
    )
}

fn console_page(
    id: PageId,
    meta_title: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    lead: &'static str,
    metrics: Vec<Metric>,
    sections: Vec<ContentSection>,
) -> PageContent {
    page(id, meta_title, lead, eyebrow, heading, lead)
        .with_metrics(metrics)
        .with_sections(sections)
        .with_local_navigation(console_navigation(id))
}

fn console_navigation(current: PageId) -> Vec<NavigationItem> {
    vec![
        nav("总览", PageId::Console, current),
        nav("资料与安全", PageId::Profile, current),
        nav("设备", PageId::Devices, current),
        nav("同步", PageId::Sync, current),
        nav("模型", PageId::Models, current),
        nav("保险库", PageId::Vault, current),
        nav("下载", PageId::ConsoleDownloads, current),
    ]
}

fn admin_page(
    id: PageId,
    meta_title: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    lead: &'static str,
) -> PageContent {
    page(id, meta_title, lead, eyebrow, heading, lead).with_local_navigation(admin_navigation(id))
}

fn admin_navigation(current: PageId) -> Vec<NavigationItem> {
    vec![
        nav("00 总览", PageId::Admin, current),
        nav("10 用户", PageId::AdminUsers, current),
        nav("20 设备", PageId::AdminDevices, current),
        nav("30 版本", PageId::AdminReleases, current),
        nav("40 资产", PageId::AdminAssets, current),
        nav("50 站点", PageId::AdminSite, current),
        nav("60 审计", PageId::AdminAudit, current),
        nav("70 反馈", PageId::AdminFeedback, current),
    ]
}
