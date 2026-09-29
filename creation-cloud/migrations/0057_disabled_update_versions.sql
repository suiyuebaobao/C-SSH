-- 追加可恢复的关闭列表；旧发布记录保持内容和修订身份。
ALTER TABLE update_policy_draft
    ADD COLUMN disabled_versions TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE update_policy_publications
    ADD COLUMN disabled_versions TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE update_policy_draft
    ADD CONSTRAINT update_policy_draft_disjoint CHECK (NOT (forced_versions && disabled_versions)),
    ADD CONSTRAINT update_policy_draft_disabled_limit CHECK (cardinality(disabled_versions) <= 128);
ALTER TABLE update_policy_publications
    ADD CONSTRAINT update_policy_publication_disjoint CHECK (NOT (forced_versions && disabled_versions)),
    ADD CONSTRAINT update_policy_publication_disabled_limit CHECK (cardinality(disabled_versions) <= 128);
