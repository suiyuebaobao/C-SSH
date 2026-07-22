-- 管理员登录名可空，普通用户继续只使用唯一邮箱注册与登录。
ALTER TABLE accounts
ADD COLUMN admin_login_name TEXT;

-- 数据库与共享值规则保持同一 ASCII 小写格式，避免绕过 CLI 写入歧义标识。
ALTER TABLE accounts
ADD CONSTRAINT accounts_admin_login_name_format_check
CHECK (
    admin_login_name IS NULL
    OR (
        length(admin_login_name) BETWEEN 3 AND 32
        AND admin_login_name = lower(admin_login_name)
        AND admin_login_name ~ '^[a-z][a-z0-9_-]{2,31}$'
    )
);

-- 登录名只能属于管理员；业务降权语句必须在同一更新中清空该字段。
ALTER TABLE accounts
ADD CONSTRAINT accounts_admin_login_name_role_check
CHECK (admin_login_name IS NULL OR role = 'admin');

CREATE UNIQUE INDEX accounts_admin_login_name_unique_idx
ON accounts (admin_login_name)
WHERE admin_login_name IS NOT NULL;
