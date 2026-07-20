CREATE TABLE audit_events (
    id UUID PRIMARY KEY,
    actor_account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    action TEXT NOT NULL,
    resource_kind TEXT NOT NULL,
    resource_id TEXT,
    outcome TEXT NOT NULL CHECK (outcome IN ('success', 'failure')),
    request_id TEXT CHECK (request_id IS NULL OR length(request_id) BETWEEN 1 AND 128),
    details JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(details) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_events_created_at_idx ON audit_events(created_at DESC);
CREATE INDEX audit_events_actor_idx ON audit_events(actor_account_id);
CREATE INDEX audit_events_resource_idx ON audit_events(resource_kind, resource_id);

CREATE FUNCTION guard_audit_event_immutability() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'audit events are immutable';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER audit_events_immutable
BEFORE UPDATE OR DELETE ON audit_events
FOR EACH ROW EXECUTE FUNCTION guard_audit_event_immutability();
