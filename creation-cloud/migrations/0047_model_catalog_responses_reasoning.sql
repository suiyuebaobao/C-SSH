-- 全局模型目录新增 provider-neutral Responses 兼容接口与强类型思考能力。
-- 既有模型不从 URL、model ID、任意标签或默认参数猜测能力，确定性回填为 unsupported。
ALTER TABLE global_model_catalog
    ADD COLUMN responses_base_url TEXT,
    ADD COLUMN responses_model_name TEXT,
    ADD COLUMN reasoning_control TEXT NOT NULL DEFAULT 'unsupported';

ALTER TABLE global_model_catalog
    DROP CONSTRAINT global_model_catalog_interface_required_check,
    ADD CONSTRAINT global_model_catalog_responses_pair_check CHECK (
        (responses_base_url IS NULL AND responses_model_name IS NULL)
        OR
        (responses_base_url IS NOT NULL AND responses_model_name IS NOT NULL)
    ),
    ADD CONSTRAINT global_model_catalog_interface_required_check CHECK (
        openai_base_url IS NOT NULL
        OR anthropic_base_url IS NOT NULL
        OR responses_base_url IS NOT NULL
    ),
    ADD CONSTRAINT global_model_catalog_responses_url_check CHECK (
        responses_base_url IS NULL OR (
            length(responses_base_url) BETWEEN 9 AND 512
            AND responses_base_url LIKE 'https://%'
            AND responses_base_url !~ '[[:cntrl:][:space:]@?#]'
        )
    ),
    ADD CONSTRAINT global_model_catalog_responses_model_name_check CHECK (
        responses_model_name IS NULL OR length(responses_model_name) BETWEEN 1 AND 128
    ),
    ADD CONSTRAINT global_model_catalog_reasoning_control_check CHECK (
        reasoning_control IN ('unsupported', 'deepseek')
    );

COMMENT ON COLUMN global_model_catalog.responses_base_url IS
    'Responses-compatible HTTPS base URL without credentials, query or fragment';
COMMENT ON COLUMN global_model_catalog.responses_model_name IS
    'Provider model ID for the Responses-compatible interface';
COMMENT ON COLUMN global_model_catalog.reasoning_control IS
    'Authoritative request-shape adapter: unsupported or deepseek; never inferred from endpoint data';
