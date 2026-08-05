-- 管理员永久删除账号时，普通用户反馈随账号级联删除；脱敏责任人仍保持 RESTRICT。
ALTER TABLE feedback_submissions
    DROP CONSTRAINT feedback_submissions_account_id_fkey,
    ADD CONSTRAINT feedback_submissions_account_id_fkey
        FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE;

-- 反馈正文仍禁止直接物理删除，仅允许 accounts 外键触发的嵌套级联删除。
CREATE OR REPLACE FUNCTION guard_feedback_submission_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF pg_trigger_depth() > 1 THEN
            RETURN OLD;
        END IF;
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

-- 管理端逻辑删除只影响历史可见性，不破坏上传幂等证据或下载 checkpoint。
ALTER TABLE cloud_host_mutations
    ADD COLUMN admin_deleted_at TIMESTAMPTZ;

ALTER TABLE cloud_host_device_checkpoints
    ADD COLUMN admin_deleted_at TIMESTAMPTZ;

CREATE INDEX cloud_host_mutations_admin_visible_idx
    ON cloud_host_mutations(account_id, created_at DESC, client_mutation_id DESC)
    WHERE admin_deleted_at IS NULL;

CREATE INDEX cloud_host_device_checkpoints_admin_visible_idx
    ON cloud_host_device_checkpoints(account_id, last_manual_sync_at DESC, device_id DESC)
    WHERE admin_deleted_at IS NULL;
