-- 追加无需更新列表；旧草稿和历史发布记录按空列表兼容。
ALTER TABLE update_policy_draft
    ADD COLUMN no_update_versions TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE update_policy_publications
    ADD COLUMN no_update_versions TEXT[] NOT NULL DEFAULT '{}';

ALTER TABLE update_policy_draft
    ADD CONSTRAINT update_policy_draft_no_update_limit
        CHECK (cardinality(no_update_versions) <= 128),
    ADD CONSTRAINT update_policy_draft_forced_no_update_disjoint
        CHECK (NOT (forced_versions && no_update_versions)),
    ADD CONSTRAINT update_policy_draft_disabled_no_update_disjoint
        CHECK (NOT (disabled_versions && no_update_versions));

ALTER TABLE update_policy_publications
    ADD CONSTRAINT update_policy_publication_no_update_limit
        CHECK (cardinality(no_update_versions) <= 128),
    ADD CONSTRAINT update_policy_publication_forced_no_update_disjoint
        CHECK (NOT (forced_versions && no_update_versions)),
    ADD CONSTRAINT update_policy_publication_disabled_no_update_disjoint
        CHECK (NOT (disabled_versions && no_update_versions));
