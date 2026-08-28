-- 既有公告在同一前进迁移中安全补为 normal，后续值只允许闭集三档。
ALTER TABLE global_announcements
ADD COLUMN priority TEXT NOT NULL DEFAULT 'normal'
CHECK (priority IN ('normal', 'important', 'critical'));

-- 发布与隐藏只能改变状态字段，不能在状态切换时旁路草稿优先级。
CREATE OR REPLACE FUNCTION guard_global_announcement_mutation() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'draft' THEN
            RAISE EXCEPTION 'only draft announcements can be deleted';
        END IF;
        RETURN OLD;
    END IF;

    IF NEW.id IS DISTINCT FROM OLD.id
        OR NEW.created_by IS DISTINCT FROM OLD.created_by
        OR NEW.created_at IS DISTINCT FROM OLD.created_at THEN
        RAISE EXCEPTION 'announcement identity is immutable';
    END IF;

    IF NEW.revision <> OLD.revision + 1
        OR NEW.updated_by IS NULL
        OR NEW.updated_at < OLD.updated_at THEN
        RAISE EXCEPTION 'invalid announcement revision';
    END IF;

    IF OLD.status = 'draft' AND NEW.status = 'draft' THEN
        IF NEW.published_at IS NOT NULL OR NEW.hidden_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid announcement draft update';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.status = 'draft' AND NEW.status = 'published' THEN
        IF NEW.title_zh_cn IS DISTINCT FROM OLD.title_zh_cn
            OR NEW.body_zh_cn IS DISTINCT FROM OLD.body_zh_cn
            OR NEW.title_en IS DISTINCT FROM OLD.title_en
            OR NEW.body_en IS DISTINCT FROM OLD.body_en
            OR NEW.priority IS DISTINCT FROM OLD.priority
            OR NEW.published_at IS NULL
            OR NEW.hidden_at IS NOT NULL THEN
            RAISE EXCEPTION 'invalid announcement publication';
        END IF;
        RETURN NEW;
    END IF;

    IF OLD.status = 'published' AND NEW.status = 'hidden' THEN
        IF NEW.title_zh_cn IS DISTINCT FROM OLD.title_zh_cn
            OR NEW.body_zh_cn IS DISTINCT FROM OLD.body_zh_cn
            OR NEW.title_en IS DISTINCT FROM OLD.title_en
            OR NEW.body_en IS DISTINCT FROM OLD.body_en
            OR NEW.priority IS DISTINCT FROM OLD.priority
            OR NEW.published_at IS DISTINCT FROM OLD.published_at
            OR NEW.hidden_at IS NULL THEN
            RAISE EXCEPTION 'invalid announcement hide';
        END IF;
        RETURN NEW;
    END IF;

    RAISE EXCEPTION 'invalid announcement state transition';
END;
$$ LANGUAGE plpgsql;
