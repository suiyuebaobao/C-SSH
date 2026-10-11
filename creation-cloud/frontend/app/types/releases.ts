export interface AdminRelease {
  latest_revision: number;
  updated_at: string;
  id: string;
  version: string;
  revision: number;
  status: string;
  channel: string;
}
export interface Policy {
  revision: number;
  enabled: boolean;
  forced_versions: string[];
  disabled_versions: string[];
  no_update_versions: string[];
  target_release_id: string | null;
  sha256_enabled: boolean;
}
export interface PolicySnapshot {
  draft: Policy;
  published: Policy;
  target_releases: {
    id: string;
    version: string;
  revision: number;
    eligible: boolean;
    readiness: string;
  }[];
}
export interface AdminAsset {
  id: string;
  release_id: string;
  platform: string;
  architecture: string;
  package_kind: string;
  file_name: string;
  byte_size: number;
  sha256: string;
}
export interface AdminSource {
  id: string;
  provider_name: string;
  source_kind: string;
  sort_order: number;
  enabled: boolean;
}
