-- 反馈只保存登录账号明确提交的纯文本，不采集 IP、UA、附件、凭据或额外邮箱。
CREATE TABLE feedback_submissions (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    request_id TEXT NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN ('bug', 'feature', 'docs', 'compatibility', 'other')),
    platform TEXT NOT NULL
        CHECK (platform IN ('windows', 'linux', 'android', 'macos', 'ios', 'cloud', 'agent', 'other')),
    app_version TEXT,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'triaged', 'in_progress', 'resolved', 'closed')),
    version BIGINT NOT NULL DEFAULT 1 CHECK (version >= 1),
    redacted_at TIMESTAMPTZ,
    redacted_by UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    redaction_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT feedback_request_id_format CHECK (
        char_length(request_id) BETWEEN 1 AND 128
        AND request_id ~ '^[A-Za-z0-9._:-]+$'
    ),
    CONSTRAINT feedback_title_format CHECK (
        char_length(title) BETWEEN 5 AND 120
        AND title = btrim(title)
        AND title !~ '[[:cntrl:]]'
    ),
    CONSTRAINT feedback_description_format CHECK (
        char_length(description) BETWEEN 20 AND 4000
        AND description = btrim(description)
        AND regexp_replace(description, E'[\\n\\r\\t]', '', 'g') !~ '[[:cntrl:]]'
    ),
    CONSTRAINT feedback_app_version_format CHECK (
        app_version IS NULL OR (
            char_length(app_version) BETWEEN 1 AND 32
            AND app_version = btrim(app_version)
            AND app_version ~ '^[A-Za-z0-9._+-]+$'
        )
    ),
    CONSTRAINT feedback_redaction_pair CHECK (
        (redacted_at IS NULL AND redacted_by IS NULL AND redaction_reason IS NULL)
        OR (
            redacted_at IS NOT NULL
            AND redacted_by IS NOT NULL
            AND redaction_reason IS NOT NULL
            AND char_length(redaction_reason) BETWEEN 5 AND 500
            AND redaction_reason = btrim(redaction_reason)
            AND redaction_reason !~ '[[:cntrl:]]'
            AND status = 'closed'
        )
    )
);

CREATE INDEX feedback_submissions_account_created_idx
ON feedback_submissions (account_id, created_at DESC, id DESC);

CREATE INDEX feedback_submissions_status_created_idx
ON feedback_submissions (status, created_at DESC, id DESC);

CREATE INDEX feedback_submissions_admin_created_idx
ON feedback_submissions (created_at DESC, id DESC);

-- 数据库层拒绝物理删除、越权改归属以及绕过状态机或乐观锁的写入。
CREATE FUNCTION guard_feedback_submission_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'feedback submissions must be retained';
    END IF;

    IF NEW.account_id IS DISTINCT FROM OLD.account_id
        OR NEW.request_id IS DISTINCT FROM OLD.request_id
        OR NEW.category IS DISTINCT FROM OLD.category
        OR NEW.platform IS DISTINCT FROM OLD.platform
        OR NEW.app_version IS DISTINCT FROM OLD.app_version
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'feedback submission identity is immutable';
    END IF;

    IF NEW.version <> OLD.version + 1 THEN
        RAISE EXCEPTION 'feedback version must advance exactly once';
    END IF;

    IF OLD.status = 'new' AND NEW.status NOT IN ('triaged', 'closed') THEN
        RAISE EXCEPTION 'invalid feedback status transition';
    ELSIF OLD.status = 'triaged' AND NEW.status NOT IN ('in_progress', 'closed') THEN
        RAISE EXCEPTION 'invalid feedback status transition';
    ELSIF OLD.status = 'in_progress' AND NEW.status NOT IN ('resolved', 'closed') THEN
        RAISE EXCEPTION 'invalid feedback status transition';
    ELSIF OLD.status = 'resolved' AND NEW.status NOT IN ('in_progress', 'closed') THEN
        RAISE EXCEPTION 'invalid feedback status transition';
    ELSIF OLD.status = 'closed' AND NEW.status <> 'closed' THEN
        RAISE EXCEPTION 'closed feedback is terminal';
    END IF;

    IF OLD.redacted_at IS NULL AND NEW.redacted_at IS NOT NULL THEN
        IF NEW.redacted_by IS NULL
            OR NEW.redaction_reason IS NULL
            OR NEW.status <> 'closed'
            OR NEW.title <> '[已由管理员安全脱敏]'
            OR NEW.description <> '[反馈正文已由管理员执行不可逆安全脱敏]' THEN
            RAISE EXCEPTION 'feedback text can only be irreversibly redacted';
        END IF;
    ELSIF NEW.title IS DISTINCT FROM OLD.title OR NEW.description IS DISTINCT FROM OLD.description THEN
        RAISE EXCEPTION 'feedback text can only be irreversibly redacted';
    END IF;

    IF OLD.redacted_at IS NOT NULL AND (
        NEW.redacted_at IS DISTINCT FROM OLD.redacted_at
        OR NEW.redacted_by IS DISTINCT FROM OLD.redacted_by
        OR NEW.redaction_reason IS DISTINCT FROM OLD.redaction_reason
    ) THEN
        RAISE EXCEPTION 'feedback redaction is irreversible';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER feedback_submissions_mutation_guard
BEFORE UPDATE OR DELETE ON feedback_submissions
FOR EACH ROW EXECUTE FUNCTION guard_feedback_submission_mutation();
