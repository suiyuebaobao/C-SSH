-- 教学媒体独立于客户端安装资产；已删除条目保留最小墓碑以继续精确清理文件。
CREATE TABLE tutorial_videos (
    id UUID PRIMARY KEY,
    locale TEXT NOT NULL CHECK (locale IN ('zh-CN', 'en')),
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 160),
    description TEXT NOT NULL CHECK (length(description) <= 4000),
    sort_order INTEGER NOT NULL CHECK (sort_order BETWEEN 0 AND 9999),
    version_note TEXT NOT NULL CHECK (length(version_note) <= 160),
    state TEXT NOT NULL DEFAULT 'draft' CHECK (state IN ('draft', 'published', 'hidden', 'deleted')),
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    video_bytes BIGINT NOT NULL CHECK (video_bytes BETWEEN 16 AND 209715200),
    video_sha256 TEXT NOT NULL CHECK (video_sha256 ~ '^[0-9a-f]{64}$'),
    poster_bytes BIGINT NOT NULL CHECK (poster_bytes BETWEEN 1 AND 67108864),
    poster_sha256 TEXT NOT NULL CHECK (poster_sha256 ~ '^[0-9a-f]{64}$'),
    created_by UUID REFERENCES accounts(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    purged_at TIMESTAMPTZ,
    CHECK (purged_at IS NULL OR state = 'deleted')
);

CREATE INDEX tutorial_videos_public_idx ON tutorial_videos(locale, sort_order, created_at, id)
    WHERE state = 'published';
CREATE INDEX tutorial_videos_admin_idx ON tutorial_videos(updated_at DESC, id)
    WHERE state <> 'deleted' OR purged_at IS NULL;

ALTER TABLE audit_events ADD CONSTRAINT audit_events_tutorial_contract CHECK (
    action NOT LIKE 'tutorial_video.%' OR (
        action IN ('tutorial_video.created', 'tutorial_video.updated', 'tutorial_video.published',
                   'tutorial_video.hidden', 'tutorial_video.deleted')
        AND resource_kind = 'tutorial_video'
        AND outcome = 'success'
        AND resource_id IS NOT NULL
        AND details ?& ARRAY['video_id', 'revision', 'state', 'locale']
        AND details - ARRAY['video_id', 'revision', 'state', 'locale'] = '{}'::jsonb
        AND jsonb_typeof(details->'revision') = 'number'
        AND details->>'state' IN ('draft', 'published', 'hidden', 'deleted')
        AND details->>'locale' IN ('zh-CN', 'en')
        AND details->>'video_id' = resource_id
    )
);
