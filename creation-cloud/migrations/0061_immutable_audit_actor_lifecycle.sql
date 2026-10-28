-- 审计保存记录时的身份，不因账号生命周期回写不可变历史。
-- 迁移由同一事务替换外键，不留可写入未知 actor 的窗口。
ALTER TABLE audit_events
    DROP CONSTRAINT audit_events_actor_account_id_fkey;

CREATE FUNCTION guard_audit_actor_at_insert() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.actor_account_id IS NOT NULL THEN
        -- 与并发账号删除互斥，锁一直持有到记录事务结束。
        PERFORM id FROM accounts
        WHERE id = NEW.actor_account_id
        FOR KEY SHARE;

        IF NOT FOUND THEN
            RAISE EXCEPTION 'audit actor account does not exist'
                USING ERRCODE = '23503',
                      CONSTRAINT = 'audit_events_actor_at_insert',
                      TABLE = 'audit_events',
                      COLUMN = 'actor_account_id';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SET search_path FROM CURRENT;

CREATE TRIGGER audit_actor_exists_at_insert
BEFORE INSERT ON audit_events
FOR EACH ROW EXECUTE FUNCTION guard_audit_actor_at_insert();

COMMENT ON COLUMN audit_events.actor_account_id IS
    'Immutable actor identity at recording time; public audit reads project missing accounts as null.';
