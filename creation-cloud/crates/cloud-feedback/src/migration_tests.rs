//! 静态验证反馈迁移的字段、约束、保留策略与索引边界。

const MIGRATION: &str = include_str!("../../../migrations/0011_feedback.sql");
const SEMANTIC_AUDIT_MIGRATION: &str =
    include_str!("../../../migrations/0023_feedback_semantic_audit.sql");

#[test]
fn migration_defines_the_final_feedback_contract() {
    for required in [
        "CREATE TABLE feedback_submissions",
        "account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT",
        "request_id TEXT NOT NULL",
        "'bug', 'feature', 'docs', 'compatibility', 'other'",
        "'windows', 'linux', 'android', 'macos', 'ios', 'cloud', 'agent', 'other'",
        "char_length(title) BETWEEN 5 AND 120",
        "char_length(description) BETWEEN 20 AND 4000",
        "char_length(app_version) BETWEEN 1 AND 32",
        "version BIGINT NOT NULL DEFAULT 1",
        "redaction_reason TEXT",
        "char_length(redaction_reason) BETWEEN 5 AND 500",
        "feedback_submissions_account_created_idx",
        "feedback_submissions_status_created_idx",
    ] {
        assert!(MIGRATION.contains(required), "迁移缺少契约: {required}");
    }
}

#[test]
fn migration_guards_retention_state_and_irreversible_redaction() {
    for required in [
        "feedback submissions must be retained",
        "feedback version must advance exactly once",
        "invalid feedback status transition",
        "closed feedback is terminal",
        "feedback text can only be irreversibly redacted",
        "feedback redaction is irreversible",
        "NEW.redaction_reason IS NULL",
        "NEW.redaction_reason IS DISTINCT FROM OLD.redaction_reason",
        "[已由管理员安全脱敏]",
        "[反馈正文已由管理员执行不可逆安全脱敏]",
    ] {
        assert!(MIGRATION.contains(required), "迁移缺少保护: {required}");
    }
}

#[test]
fn migration_does_not_add_ip_ua_attachment_or_email_columns() {
    let lower = MIGRATION.to_ascii_lowercase();
    for forbidden in [
        "ip_address ",
        "user_agent ",
        "attachment ",
        "contact_email ",
    ] {
        assert!(!lower.contains(forbidden), "迁移不得定义字段: {forbidden}");
    }
}

#[test]
fn semantic_audit_migration_enforces_fixed_action_contracts() {
    for required in [
        "audit_events_feedback_semantic_contract",
        "feedback.status_changed",
        "feedback.redacted",
        "record_feedback_semantic_audit",
        "unsupported feedback semantic audit action",
        "from_status",
        "to_status",
        "reason_summary",
        "failure_code",
        "invalid_transition",
        "version_conflict",
    ] {
        assert!(
            SEMANTIC_AUDIT_MIGRATION.contains(required),
            "语义审计迁移缺少契约: {required}"
        );
    }
}

#[test]
fn semantic_audit_contract_rejects_extra_or_identity_fields() {
    assert!(SEMANTIC_AUDIT_MIGRATION.contains("details - ARRAY["));
    assert!(SEMANTIC_AUDIT_MIGRATION.contains("[[:cntrl:]@]"));
    for forbidden in ["'title'", "'description'", "'email'", "'account_email'"] {
        assert!(
            !SEMANTIC_AUDIT_MIGRATION.contains(forbidden),
            "语义审计迁移不得允许字段: {forbidden}"
        );
    }
}
