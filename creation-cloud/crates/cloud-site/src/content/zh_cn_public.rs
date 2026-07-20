//! 提供简体中文安全、下载、更新与常见问题内容。

use crate::{FaqItem, PageContent, PageId};

use super::zh_cn::{action, item, page, section};

pub(super) fn security() -> PageContent {
    page(
        PageId::Security,
        "SSH 隧道、主机密钥与保险库安全｜Creation-SSH",
        "了解 Creation-SSH 的 SSH 隧道、主机密钥校验、agent 本地 socket、云端数据边界与客户端加密保险库。",
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
                    "非敏感白名单",
                    "只同步明确允许的偏好与模型元数据。",
                    "默认拒绝",
                ),
                item(
                    "保险库",
                    "客户端加密",
                    "服务端仅保存密文信封和密钥包装元数据。",
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
        "客户端发布与平台状态｜Creation-SSH",
        "查看 Creation-SSH 的 Windows、Linux、Android、macOS 与 iOS 平台状态；真实发布后可核对来源、架构与 SHA256。",
        "版本信息以发布记录为准",
        "为你的平台选择正确构建",
        "正式资产接入后，这里会同时展示本站来源与明确标注的外部镜像。",
    )
    .with_actions(vec![action(
        "查看更新记录",
        "/changelog",
        "button button-secondary",
    )])
    .with_sections(vec![section(
        "builds",
        "平台构建",
        "页面不会用占位链接冒充可下载资产。",
        vec![
            item(
                "桌面",
                "Windows",
                "安装包、MSI 与便携版将分别列出文件大小和 SHA256。",
                "等待发布数据",
            ),
            item(
                "桌面",
                "Linux",
                "AppImage 与 deb 只展示完成本地构建和真实验证的版本。",
                "等待发布数据",
            ),
            item(
                "移动",
                "Android",
                "正式 arm64 APK 与 AAB 和仅测试用 x86_64 构建严格区分。",
                "等待发布数据",
            ),
            item(
                "规划中",
                "macOS",
                "独立 macOS 客户端尚未开发，当前不提供下载。",
                "尚未开发",
            ),
            item(
                "规划中",
                "iOS",
                "独立 iOS 移动伴侣尚未开发，当前不提供下载。",
                "尚未开发",
            ),
        ],
    )])
}

pub(super) fn changelog() -> PageContent {
    page(
        PageId::Changelog,
        "版本记录与发布原则｜Creation-SSH",
        "查看 Creation-SSH 的版本记录、资产来源、SHA256 与真实发布验证原则；当前不使用模拟版本数据。",
        "不可覆盖的发布历史",
        "每个正式版本都有自己的记录",
        "发布数据接入后，版本说明、资产来源、验证结果与 SHA256 将在同一条目中呈现。",
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
        "SSH 客户端与 agent 常见问题｜Creation-SSH",
        "解答 Creation-SSH 的 SSH 连接、常驻 agent、云同步、凭据隐私、安装包校验和移动端定位问题。",
        "常见问题",
        "先把关键边界说清楚",
        "关于连接方式、agent、云同步与下载版本的简明回答。",
    )
    .with_faqs(vec![
        FaqItem::new("Creation Cloud 会代理 SSH 连接吗？", "不会。SSH 数据面保持客户端直连用户服务器，云端只做账号、设备与可选同步等控制面能力。"),
        FaqItem::new("没有安装 agent 还能使用吗？", "普通 SSH 终端和端口映射当前可走原生 SSH；跳板机属于同类架构例外，但本阶段仍延期。持久会话、监控和结构化管理能力依赖 agent。"),
        FaqItem::new("主机地址和私钥会同步到云端吗？", "不会明文同步。主机资料、密码、私钥、known_hosts、终端内容和命令历史都不属于允许上云的数据。"),
        FaqItem::new("保险库密码和账号密码相同吗？", "不同。账号密码只负责登录；保险库密码只在可信客户端派生加密密钥，不上传服务端。"),
        FaqItem::new("如何确认下载文件未被替换？", "正式下载条目会展示平台、架构、文件大小与 SHA256；请在安装前核对。"),
        FaqItem::new("移动端是桌面端的完整复制吗？", "不是。Android 定位为移动伴侣，优先覆盖查看、轻量操作和与桌面工作流衔接的场景。"),
    ])
}
