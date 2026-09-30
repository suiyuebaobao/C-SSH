//! 验证维护 CLI 直接使用的公开 DTO 保持可序列化边界。

use super::*;

fn assert_serializable<T: Serialize>() {}

#[test]
fn cli_dtos_are_json_serializable_without_custom_projection() {
    assert_serializable::<cloud_maintenance::RunRecord>();
    assert_serializable::<cloud_maintenance::MaintenanceStatus>();
}
