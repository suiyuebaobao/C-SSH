// 仅投影发布表单选项与已发布下载目录；实际发布权限和闭合集合由服务端校验。
import type { Platform, Release } from "~/types/site";

export function manualThreePlatformRelease(version: string): boolean {
  const match = /^(\d+)\.(\d+)\.(\d+)(?:[-+]|$)/.exec(version);
  if (!match) return false;
  const [major, minor, patch] = match.slice(1).map(Number);
  return major! > 0 || minor! > 9 || (minor === 9 && patch! >= 9);
}

export function releaseFormFormats(version: string) {
  const manual = manualThreePlatformRelease(version);
  const formats: Record<string, { name: string; architectures: string[]; packages: string[] }> = {
    windows: { name: "Windows", architectures: manual ? ["x86_64"] : ["x86_64", "aarch64"], packages: manual ? ["zip"] : ["exe", "zip"] },
    macos: { name: "macOS", architectures: ["aarch64"], packages: ["dmg"] },
    android: { name: "Android", architectures: ["aarch64"], packages: ["apk"] },
  };
  if (!manual) formats.linux = { name: "Linux", architectures: ["x86_64", "aarch64"], packages: ["deb", "appimage"] };
  return formats;
}

export type DownloadPlatform = Platform & { note?: string; version: string; revision: number };

/** 输入沿服务端已发布版本顺序；平台未更新时保留其最近版本，不拼接旧Windows安装包。 */
export function latestDownloadPlatforms(releases: readonly Release[]): DownloadPlatform[] {
  const result: DownloadPlatform[] = [];
  for (const release of releases) {
    const platforms: DownloadPlatform[] = [];
    for (const platform of [...release.platforms, ...release.manual_platforms]) {
      const existing = platforms.find(item => item.name === platform.name);
      if (!existing) {
        platforms.push({ ...platform, assets: [...platform.assets], version: release.version, revision: release.revision });
        continue;
      }
      for (const asset of platform.assets) {
        if (!existing.assets.some(item => item.architecture === asset.architecture && item.package_kind === asset.package_kind)) existing.assets.push(asset);
      }
      if ("note" in platform && typeof platform.note === "string") existing.note = platform.note;
    }
    for (const platform of platforms) {
      if (!result.some(item => item.name === platform.name)) result.push(platform);
    }
  }
  return result;
}
