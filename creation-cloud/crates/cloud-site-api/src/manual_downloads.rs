//! 投影已发布版本的官网补充分发入口；不扩展客户端自动更新资产合同。

use super::catalog::{PublishedAssetView, PublishedPlatformView};
use cloud_site::Locale;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ManualPlatform {
    pub name: String,
    pub assets: Vec<PublishedAssetView>,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ManualNotice {
    pub title: String,
    pub body: String,
    pub link: cloud_site::EditableLink,
}

pub fn notice(version: &str, locale: Locale) -> Option<ManualNotice> {
    if !matches!(version, "0.9.0" | "0.9.1" | "0.9.2" | "0.9.3" | "0.9.4") {
        return None;
    }
    let en = locale == Locale::En;
    let title = if version == "0.9.0" {
        if en {
            "0.9.0 must be downloaded and installed manually. Automatic updates from older versions are unavailable.".into()
        } else {
            "0.9.0 必须自行下载并安装，旧版本无法自动更新。".into()
        }
    } else if en {
        format!(
            "{version} is announced here for manual download and installation. No upgrade reminders or forced updates are sent."
        )
    } else {
        format!("{version} 仅通过公告告知，请手动下载安装；不推送升级提醒，不强制更新。")
    };
    Some(ManualNotice {
        title,
        body: if en {
            "An older Android installation with a different signature cannot be replaced directly. Uninstalling clears local data; preserve it first, especially AI conversations and memory, which are not included in Cloud sync."
        } else {
            "不同签名的 Android 旧版不能直接覆盖。卸载会清除本地数据，请先确认数据已保全；AI 对话与记忆不包含在 Cloud 同步中。"
        }.into(),
        link: cloud_site::EditableLink {
            label: if en { "Read installation and data-preservation instructions" } else { "阅读安装与数据保全说明" }.into(),
            href: format!("https://github.com/suiyuebaobao/C-SSH/blob/main/INSTALL{}.md", if en { "_EN" } else { "" }),
        },
    })
}

pub fn platforms(
    version: &str,
    locale: Locale,
    published: &[PublishedPlatformView],
) -> Vec<ManualPlatform> {
    if !matches!(version, "0.9.0" | "0.9.1" | "0.9.2" | "0.9.3" | "0.9.4") {
        return Vec::new();
    }
    let en = locale == Locale::En;
    let mut result = Vec::new();
    if !published.iter().any(|p| p.name == "macOS") {
        result.push(ManualPlatform {
            name: "macOS".into(),
            assets: vec![asset(
                version,
                "DMG",
                "Apple Silicon · macOS 13+",
                "macOS-arm64.dmg",
            )],
            note: if en {
                "Ad-hoc signed, not notarized. No Intel build or automatic updater."
            } else {
                "仅 M 系列，ad-hoc 签名，未经公证；不提供 Intel 版本或自动更新。"
            }
            .into(),
        });
    }
    if matches!(version, "0.9.2" | "0.9.3" | "0.9.4") {
        if !published
            .iter()
            .any(|p| p.name == "Windows" && p.assets.iter().any(|a| a.architecture == "arm64"))
        {
            result.push(ManualPlatform {
                name: "Windows ARM64".into(),
                assets: vec![asset(version, "NSIS", "Windows 11 ARM64", "arm64-setup.exe"), asset(version, "Portable ZIP", "Windows 11 ARM64", "portable-Windows-arm64.zip")],
                note: if en { "Native ARM64 client. Install and update manually; choose the x64 downloads above for x64 Windows." }
                else { "原生 ARM64 客户端，手动安装与更新；x64 Windows 请使用 x64 下载。" }.into(),
            });
        }
        if !published.iter().any(|p| p.name == "Linux") {
            result.push(ManualPlatform {
                name: "Linux".into(),
                assets: vec![asset(version, "deb", "x86_64 / amd64", "amd64.deb"), asset(version, "AppImage", "x86_64 / amd64", "amd64.AppImage"), asset(version, "deb", "ARM64 / aarch64", "arm64.deb"), asset(version, "AppImage", "ARM64 / aarch64", "aarch64.AppImage")],
                note: if en { "Verified on Ubuntu 24.04 with X11. Requires an unlocked desktop Secret Service keyring; AppImage also needs FUSE support. Other distributions and Wayland are not covered by this verification." }
                else { "已在 Ubuntu 24.04／X11 验证。需要已解锁的桌面 Secret Service 密钥环，AppImage 还需 FUSE 支持；本轮未覆盖其他发行版和 Wayland。" }.into(),
            });
        }
    }
    result
}

fn asset(version: &str, package: &str, architecture: &str, suffix: &str) -> PublishedAssetView {
    PublishedAssetView {
        architecture: architecture.into(),
        package_kind: package.into(),
        download_href: Some(format!(
            "https://github.com/suiyuebaobao/C-SSH/releases/download/v{version}/C-SSH_{version}_{suffix}"
        )),
    }
}
