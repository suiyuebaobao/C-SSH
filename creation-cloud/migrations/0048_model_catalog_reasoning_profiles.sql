-- 保留 reasoning_control 字段，供滚动升级和旧 DTO 反序列化兼容。
-- 从本迁移起，接口格式是唯一运行时请求契约；厂商 Profile 不再参与配置或请求塑形。
ALTER TABLE global_model_catalog
    DROP CONSTRAINT global_model_catalog_reasoning_control_check;

-- 0047 只可能留下 unsupported/deepseek；同时覆盖候选程序曾写入的其它兼容值。
-- 不按 seed ID、厂商、URL 或 model ID 猜测，所有非 unsupported 行采用同一确定性前滚。
UPDATE global_model_catalog
SET reasoning_control = 'unsupported',
    revision = revision + 1,
    updated_at = GREATEST(now(), updated_at + interval '1 microsecond')
WHERE reasoning_control IS DISTINCT FROM 'unsupported';

ALTER TABLE global_model_catalog
    ADD CONSTRAINT global_model_catalog_reasoning_control_check CHECK (
        reasoning_control = 'unsupported'
    );

COMMENT ON COLUMN global_model_catalog.reasoning_control IS
    'Compatibility-only metadata; fixed to unsupported and ignored by runtime request shaping';
