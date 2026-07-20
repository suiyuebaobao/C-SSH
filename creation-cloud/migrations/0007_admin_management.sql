-- 管理员精确邮箱检索使用统一小写身份，并在数据库层拒绝大小写重复账号。
CREATE UNIQUE INDEX accounts_email_lower_unique_idx ON accounts (lower(email));

-- 管理用户筛选和“最后一个有效管理员”行锁按当前二元角色模型加速。
CREATE INDEX accounts_admin_management_idx
ON accounts (role, status, created_at DESC, id DESC);

CREATE INDEX accounts_active_admin_lock_idx
ON accounts (id)
WHERE role = 'admin' AND status = 'active';

-- 管理设备分页只读取控制面元数据，不新增任何 SSH 主机字段。
CREATE INDEX devices_admin_management_idx
ON devices (platform, revoked_at, created_at DESC, id DESC);

CREATE INDEX devices_admin_account_management_idx
ON devices (account_id, revoked_at, created_at DESC, id DESC);

-- 只读审计常按动作和结果定位，事件本身仍由既有触发器保持不可变。
CREATE INDEX audit_events_admin_action_idx
ON audit_events (action, outcome, created_at DESC, id DESC);
