//! 静态锁定站点媒体迁移中的发布唯一性、状态机和受控路径约束。

const MIGRATION: &str = include_str!("../../../migrations/0008_site_media.sql");

#[test]
fn migration_enforces_one_published_home_qr_and_controlled_png_identity() {
    assert!(MIGRATION.contains("WHERE state = 'published'"));
    assert!(MIGRATION.contains("storage_key ~ '^objects/[0-9a-f]{2}/"));
    assert!(MIGRATION.contains("substring(storage_key FROM 9 FOR 2)"));
    assert!(MIGRATION.contains("content_type = 'image/png'"));
    assert!(MIGRATION.contains("byte_size BETWEEN 1 AND 2097152"));
    assert!(MIGRATION.contains("width = height"));
    assert!(MIGRATION.contains("alt_zh !~ '[[:cntrl:]]'"));
}

#[test]
fn migration_enforces_forward_only_state_changes_and_draft_deletion() {
    assert!(MIGRATION.contains("OLD.state = 'draft'"));
    assert!(MIGRATION.contains("NEW.state NOT IN ('draft', 'published')"));
    assert!(MIGRATION.contains("OLD.state = 'published'"));
    assert!(MIGRATION.contains("NEW.state NOT IN ('published', 'revoked')"));
    assert!(MIGRATION.contains("OLD.state = 'revoked' AND NEW IS DISTINCT FROM OLD"));
    assert!(MIGRATION.contains("IF OLD.state <> 'draft'"));
}
