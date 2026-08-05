-- 全局模型目录明确区分 OpenAI 兼容与 Anthropic 兼容接口。
-- system_seeded 只标记由服务端初始目录补种的记录；创建人与更新人仍必须引用真实账号。
ALTER TABLE global_model_catalog
    ADD COLUMN api_format TEXT NOT NULL DEFAULT 'openai_compatible',
    ADD COLUMN system_seeded BOOLEAN NOT NULL DEFAULT FALSE,
    ADD CONSTRAINT global_model_catalog_api_format_check
        CHECK (api_format IN ('openai_compatible', 'anthropic_compatible'));
