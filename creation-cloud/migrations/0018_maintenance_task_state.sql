CREATE TABLE maintenance_task_runs (
    run_id UUID PRIMARY KEY,
    task_name TEXT NOT NULL CHECK (task_name IN (
        'expired-sessions',
        'sync-retention',
        'download-aggregation',
        'published-asset-inspection',
        'backup-freshness'
    )),
    trigger_kind TEXT NOT NULL CHECK (trigger_kind IN ('startup', 'scheduled', 'manual')),
    instance_id UUID NOT NULL,
    outcome TEXT NOT NULL CHECK (outcome IN (
        'running',
        'succeeded',
        'failed',
        'timed_out',
        'cancelled',
        'skipped_locked',
        'interrupted'
    )),
    observation_code TEXT CHECK (observation_code IS NULL OR observation_code IN (
        'healthy',
        'missing',
        'stale',
        'invalid',
        'issues_detected'
    )),
    error_code TEXT CHECK (error_code IS NULL OR error_code IN (
        'task_failed',
        'timed_out',
        'cancelled',
        'lock_held',
        'interrupted'
    )),
    cutoff_at TIMESTAMPTZ,
    active_cutoff_at TIMESTAMPTZ,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    examined_count BIGINT NOT NULL DEFAULT 0 CHECK (examined_count >= 0),
    changed_count BIGINT NOT NULL DEFAULT 0 CHECK (changed_count >= 0),
    healthy_count BIGINT NOT NULL DEFAULT 0 CHECK (healthy_count >= 0),
    issue_count BIGINT NOT NULL DEFAULT 0 CHECK (issue_count >= 0),
    UNIQUE (task_name, run_id),
    CHECK (
        (outcome = 'running' AND finished_at IS NULL AND error_code IS NULL)
        OR
        (outcome = 'succeeded' AND finished_at IS NOT NULL AND error_code IS NULL)
        OR
        (outcome IN ('failed', 'timed_out', 'cancelled', 'skipped_locked', 'interrupted')
            AND finished_at IS NOT NULL AND error_code IS NOT NULL)
    ),
    CHECK (observation_code IS NULL OR outcome = 'succeeded')
);

CREATE INDEX maintenance_task_runs_history_idx
    ON maintenance_task_runs(task_name, started_at DESC, run_id DESC)
    WHERE outcome <> 'running';

CREATE TABLE maintenance_task_state (
    task_name TEXT PRIMARY KEY CHECK (task_name IN (
        'expired-sessions',
        'sync-retention',
        'download-aggregation',
        'published-asset-inspection',
        'backup-freshness'
    )),
    active_run_id UUID,
    last_success_at TIMESTAMPTZ,
    consecutive_failures BIGINT NOT NULL DEFAULT 0 CHECK (consecutive_failures >= 0),
    last_observation_code TEXT CHECK (last_observation_code IS NULL OR last_observation_code IN (
        'healthy',
        'missing',
        'stale',
        'invalid',
        'issues_detected'
    )),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (task_name, active_run_id)
        REFERENCES maintenance_task_runs(task_name, run_id)
        ON UPDATE RESTRICT
        ON DELETE RESTRICT
);

INSERT INTO maintenance_task_state (task_name) VALUES
    ('expired-sessions'),
    ('sync-retention'),
    ('download-aggregation'),
    ('published-asset-inspection'),
    ('backup-freshness');

CREATE FUNCTION guard_maintenance_task_state_identity() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' OR NEW.task_name IS DISTINCT FROM OLD.task_name THEN
        RAISE EXCEPTION 'maintenance task state identity is immutable';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER maintenance_task_state_identity_guard
BEFORE UPDATE OF task_name OR DELETE ON maintenance_task_state
FOR EACH ROW EXECUTE FUNCTION guard_maintenance_task_state_identity();
