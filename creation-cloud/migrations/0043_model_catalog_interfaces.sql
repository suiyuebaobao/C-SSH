-- 全局模型目录从单接口前进为 OpenAI/Anthropic 两个独立接口。
-- 旧接口先确定性回填；任何缺少 base_url 的旧记录都会使迁移失败回滚，
-- 禁止为了通过约束而持久化伪造或占位接口事实。
ALTER TABLE global_model_catalog
    ADD COLUMN openai_base_url TEXT,
    ADD COLUMN openai_model_name TEXT,
    ADD COLUMN anthropic_base_url TEXT,
    ADD COLUMN anthropic_model_name TEXT;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM global_model_catalog WHERE base_url IS NULL) THEN
        RAISE EXCEPTION
            '0043_model_catalog_interfaces: legacy model is missing base_url';
    END IF;
END
$$;

UPDATE global_model_catalog
SET openai_base_url = CASE
        WHEN api_format = 'openai_compatible'
            THEN base_url
        ELSE NULL
    END,
    openai_model_name = CASE
        WHEN api_format = 'openai_compatible' THEN model_name
        ELSE NULL
    END,
    anthropic_base_url = CASE
        WHEN api_format = 'anthropic_compatible'
            THEN base_url
        ELSE NULL
    END,
    anthropic_model_name = CASE
        WHEN api_format = 'anthropic_compatible' THEN model_name
        ELSE NULL
    END;

ALTER TABLE global_model_catalog
    DROP CONSTRAINT global_model_catalog_api_format_check,
    DROP COLUMN api_format,
    DROP COLUMN base_url,
    DROP COLUMN model_name,
    DROP CONSTRAINT global_model_catalog_context_length_check,
    ADD CONSTRAINT global_model_catalog_context_length_check
        CHECK (context_length BETWEEN 4096 AND 2000000),
    ADD CONSTRAINT global_model_catalog_openai_pair_check CHECK (
        (openai_base_url IS NULL AND openai_model_name IS NULL)
        OR
        (openai_base_url IS NOT NULL AND openai_model_name IS NOT NULL)
    ),
    ADD CONSTRAINT global_model_catalog_anthropic_pair_check CHECK (
        (anthropic_base_url IS NULL AND anthropic_model_name IS NULL)
        OR
        (anthropic_base_url IS NOT NULL AND anthropic_model_name IS NOT NULL)
    ),
    ADD CONSTRAINT global_model_catalog_interface_required_check CHECK (
        openai_base_url IS NOT NULL OR anthropic_base_url IS NOT NULL
    ),
    ADD CONSTRAINT global_model_catalog_openai_url_check CHECK (
        openai_base_url IS NULL OR (
            length(openai_base_url) BETWEEN 9 AND 512
            AND openai_base_url LIKE 'https://%'
            AND openai_base_url !~ '[[:cntrl:][:space:]@?#]'
        )
    ),
    ADD CONSTRAINT global_model_catalog_anthropic_url_check CHECK (
        anthropic_base_url IS NULL OR (
            length(anthropic_base_url) BETWEEN 9 AND 512
            AND anthropic_base_url LIKE 'https://%'
            AND anthropic_base_url !~ '[[:cntrl:][:space:]@?#]'
        )
    ),
    ADD CONSTRAINT global_model_catalog_openai_model_name_check CHECK (
        openai_model_name IS NULL OR length(openai_model_name) BETWEEN 1 AND 128
    ),
    ADD CONSTRAINT global_model_catalog_anthropic_model_name_check CHECK (
        anthropic_model_name IS NULL OR length(anthropic_model_name) BETWEEN 1 AND 128
    );

COMMENT ON COLUMN global_model_catalog.openai_base_url IS
    'OpenAI-compatible HTTPS base URL without credentials, query or fragment';
COMMENT ON COLUMN global_model_catalog.openai_model_name IS
    'Provider model ID for the OpenAI-compatible interface';
COMMENT ON COLUMN global_model_catalog.anthropic_base_url IS
    'Anthropic-compatible HTTPS base URL without credentials, query or fragment';
COMMENT ON COLUMN global_model_catalog.anthropic_model_name IS
    'Provider model ID for the Anthropic-compatible interface';
