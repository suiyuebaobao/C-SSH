-- 为用户中心下载历史补充按账号倒序读取索引。
-- 公开下载继续允许 account_id 为空，只有受保护入口写入账号归属。

CREATE INDEX download_events_account_history_idx
    ON download_events(account_id, occurred_at DESC, id DESC)
    WHERE account_id IS NOT NULL;
