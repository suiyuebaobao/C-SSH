-- 反馈状态与脱敏审计只接受固定字段，禁止标题、正文、邮箱或任意扩展字段进入审计。
ALTER TABLE audit_events
ADD CONSTRAINT audit_events_feedback_semantic_contract CHECK (
    action NOT IN ('feedback.status_changed', 'feedback.redacted')
    OR (
        actor_account_id IS NOT NULL
        AND resource_kind = 'feedback'
        AND resource_id IS NOT NULL
        AND resource_id ~* '^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'
        AND request_id IS NOT NULL
        AND request_id ~ '^[A-Za-z0-9._:-]{1,128}$'
        AND details->>'feedback_id' = resource_id
        AND (
            (
                action = 'feedback.status_changed'
                AND details ?& ARRAY[
                    'feedback_id', 'from_status', 'to_status', 'reason', 'failure_code'
                ]
                AND details - ARRAY[
                    'feedback_id', 'from_status', 'to_status', 'reason', 'failure_code'
                ] = '{}'::jsonb
                AND jsonb_typeof(details->'feedback_id') = 'string'
                AND jsonb_typeof(details->'from_status') IN ('string', 'null')
                AND jsonb_typeof(details->'to_status') = 'string'
                AND details->>'to_status' IN (
                    'new', 'triaged', 'in_progress', 'resolved', 'closed'
                )
                AND jsonb_typeof(details->'reason') = 'string'
                AND char_length(details->>'reason') BETWEEN 5 AND 500
                AND details->>'reason' = btrim(details->>'reason')
                AND details->>'reason' !~ '[[:cntrl:]@]'
                AND (
                    (
                        outcome = 'success'
                        AND details->>'from_status' IN (
                            'new', 'triaged', 'in_progress', 'resolved', 'closed'
                        )
                        AND details->'failure_code' = 'null'::jsonb
                    )
                    OR (
                        outcome = 'failure'
                        AND jsonb_typeof(details->'failure_code') = 'string'
                        AND details->>'failure_code' IN (
                            'not_found', 'version_conflict', 'invalid_transition'
                        )
                        AND (
                            (
                                details->>'failure_code' = 'not_found'
                                AND details->'from_status' = 'null'::jsonb
                            )
                            OR (
                                details->>'failure_code' <> 'not_found'
                                AND details->>'from_status' IN (
                                    'new', 'triaged', 'in_progress', 'resolved', 'closed'
                                )
                            )
                        )
                    )
                )
            )
            OR (
                action = 'feedback.redacted'
                AND details ?& ARRAY['feedback_id', 'reason_summary', 'failure_code']
                AND details - ARRAY['feedback_id', 'reason_summary', 'failure_code'] = '{}'::jsonb
                AND jsonb_typeof(details->'feedback_id') = 'string'
                AND jsonb_typeof(details->'reason_summary') = 'string'
                AND char_length(details->>'reason_summary') BETWEEN 5 AND 500
                AND details->>'reason_summary' = btrim(details->>'reason_summary')
                AND details->>'reason_summary' !~ '[[:cntrl:]@]'
                AND (
                    (
                        outcome = 'success'
                        AND details->'failure_code' = 'null'::jsonb
                    )
                    OR (
                        outcome = 'failure'
                        AND jsonb_typeof(details->'failure_code') = 'string'
                        AND details->>'failure_code' IN (
                            'not_found', 'version_conflict', 'already_redacted'
                        )
                    )
                )
            )
        )
    )
);

-- 业务域只调用该受约束入口，不直接读写审计表；任何审计失败都会中止所在事务。
CREATE FUNCTION record_feedback_semantic_audit(
    event_id UUID,
    actor_id UUID,
    audit_action TEXT,
    feedback_id UUID,
    audit_outcome TEXT,
    audit_request_id TEXT,
    audit_details JSONB
) RETURNS VOID AS $$
BEGIN
    IF audit_action IS NULL
        OR audit_action NOT IN ('feedback.status_changed', 'feedback.redacted') THEN
        RAISE EXCEPTION 'unsupported feedback semantic audit action'
            USING ERRCODE = 'CCAU1';
    END IF;

    INSERT INTO audit_events (
        id, actor_account_id, action, resource_kind, resource_id,
        outcome, request_id, details
    )
    VALUES (
        event_id, actor_id, audit_action, 'feedback', feedback_id::TEXT,
        audit_outcome, audit_request_id, audit_details
    );
END;
$$ LANGUAGE plpgsql VOLATILE;
