-- Host 业务 metadata 只可由持有 CDK 的客户端迁移为不透明密文。
ALTER TABLE cloud_host_sync_states
    DROP CONSTRAINT cloud_host_sync_states_minimum_sync_contract_version_check,
    ADD CONSTRAINT cloud_host_sync_states_minimum_sync_contract_version_check
        CHECK (minimum_sync_contract_version IN (2, 3, 4));

ALTER TABLE cloud_hosts
    ALTER COLUMN address DROP NOT NULL,
    ALTER COLUMN port DROP NOT NULL,
    ALTER COLUMN name DROP NOT NULL,
    ALTER COLUMN platform DROP NOT NULL,
    ALTER COLUMN tags DROP NOT NULL,
    ALTER COLUMN status DROP NOT NULL,
    ADD COLUMN metadata_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    ADD CONSTRAINT cloud_hosts_metadata_encryption_shape CHECK (
        (NOT metadata_encrypted
            AND address IS NOT NULL AND port IS NOT NULL AND name IS NOT NULL
            AND platform IS NOT NULL AND tags IS NOT NULL AND status IS NOT NULL)
        OR
        (metadata_encrypted
            AND address IS NULL AND port IS NULL AND name IS NULL
            AND platform IS NULL AND tags IS NULL AND status IS NULL
            AND ((is_deleted AND ciphertext IS NULL)
                 OR (NOT is_deleted AND ciphertext IS NOT NULL)))
    );

ALTER TABLE cloud_host_versions
    ALTER COLUMN address DROP NOT NULL,
    ALTER COLUMN port DROP NOT NULL,
    ALTER COLUMN name DROP NOT NULL,
    ALTER COLUMN platform DROP NOT NULL,
    ALTER COLUMN tags DROP NOT NULL,
    ALTER COLUMN status DROP NOT NULL,
    ADD COLUMN metadata_encrypted BOOLEAN NOT NULL DEFAULT FALSE,
    ADD CONSTRAINT cloud_host_versions_metadata_encryption_shape CHECK (
        (NOT metadata_encrypted
            AND address IS NOT NULL AND port IS NOT NULL AND name IS NOT NULL
            AND platform IS NOT NULL AND tags IS NOT NULL AND status IS NOT NULL)
        OR
        (metadata_encrypted
            AND address IS NULL AND port IS NULL AND name IS NULL
            AND platform IS NULL AND tags IS NULL AND status IS NULL
            AND ((is_deleted AND ciphertext IS NULL)
                 OR (NOT is_deleted AND ciphertext IS NOT NULL)))
    );

CREATE TABLE cloud_host_metadata_migrations (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    request_generation BIGINT NOT NULL CHECK (request_generation > 0),
    request_protection_epoch BIGINT NOT NULL CHECK (request_protection_epoch > 0),
    request_protection_revision BIGINT NOT NULL CHECK (request_protection_revision > 0),
    request_snapshot_revision BIGINT NOT NULL CHECK (request_snapshot_revision >= 0),
    result_generation BIGINT NOT NULL CHECK (result_generation = request_generation + 1),
    result_protection_epoch BIGINT NOT NULL,
    result_protection_revision BIGINT NOT NULL,
    result_current_revision BIGINT NOT NULL CHECK (result_current_revision >= 0),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    host_count INTEGER NOT NULL CHECK (host_count >= 0),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (account_id, mutation_id),
    FOREIGN KEY (account_id, source_device_id)
        REFERENCES devices(account_id, id) ON DELETE RESTRICT,
    CHECK (result_protection_epoch = request_protection_epoch),
    CHECK (result_protection_revision = request_protection_revision + 1)
);

CREATE UNIQUE INDEX cloud_host_metadata_migrations_generation_idx
    ON cloud_host_metadata_migrations(account_id, result_generation);

CREATE TABLE cloud_host_metadata_migration_results (
    account_id UUID NOT NULL,
    mutation_id UUID NOT NULL,
    host_id UUID NOT NULL,
    previous_revision BIGINT NOT NULL CHECK (previous_revision > 0),
    result_revision BIGINT NOT NULL CHECK (result_revision > previous_revision),
    PRIMARY KEY (account_id, mutation_id, host_id),
    FOREIGN KEY (account_id, mutation_id)
        REFERENCES cloud_host_metadata_migrations(account_id, mutation_id)
        ON DELETE CASCADE
);

ALTER TABLE cloud_sync_push_mutations
    ADD COLUMN request_hash_scheme TEXT;

-- 0045以前导入的历史 mutation 没有 typed resource result，原请求形状也与
-- v2/v3 PushRequest 不同；保留其摘要但明确标为不可按当前算法复算。
UPDATE cloud_sync_push_mutations AS mutations
SET request_hash_scheme = CASE
    WHEN EXISTS (
        SELECT 1 FROM cloud_sync_push_results AS results
        WHERE results.account_id = mutations.account_id
          AND results.client_mutation_id = mutations.client_mutation_id
    ) THEN 'legacy_server_struct_v1'
    ELSE 'legacy_unreplayable_v0'
END;

ALTER TABLE cloud_sync_push_mutations
    ALTER COLUMN request_hash_scheme SET DEFAULT 'legacy_server_struct_v1',
    ALTER COLUMN request_hash_scheme SET NOT NULL,
    ADD CONSTRAINT cloud_sync_push_mutations_request_hash_scheme_check CHECK (
        request_hash_scheme IN (
            'legacy_unreplayable_v0',
            'legacy_server_struct_v1',
            'canonical_json_v1'
        )
    );

ALTER TABLE cloud_data_protection_mutations
    ADD COLUMN request_hash_scheme TEXT NOT NULL DEFAULT 'data_protection_server_struct_v1'
        CHECK (request_hash_scheme IN (
            'data_protection_server_struct_v1',
            'canonical_json_v1'
        ));

CREATE TABLE cloud_sync_push_receipts (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    client_mutation_id UUID NOT NULL,
    source_device_id UUID NOT NULL,
    request_generation BIGINT NOT NULL CHECK (request_generation > 0),
    request_protection_epoch BIGINT NOT NULL CHECK (request_protection_epoch >= 0),
    request_protection_revision BIGINT NOT NULL CHECK (request_protection_revision >= 0),
    request_hash BYTEA NOT NULL CHECK (octet_length(request_hash) = 32),
    request_hash_scheme TEXT NOT NULL CHECK (
        request_hash_scheme IN (
            'legacy_unreplayable_v0',
            'legacy_server_struct_v1',
            'canonical_json_v1'
        )
    ),
    outcome TEXT NOT NULL CHECK (outcome IN ('applied', 'unchanged')),
    result_revision BIGINT NOT NULL CHECK (result_revision >= 0),
    changed_count INTEGER NOT NULL CHECK (changed_count >= 0),
    created_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (account_id, client_mutation_id),
    CHECK (
        (outcome = 'applied' AND changed_count > 0)
        OR (outcome = 'unchanged' AND changed_count = 0)
    )
);

INSERT INTO cloud_sync_push_receipts
    (account_id,client_mutation_id,source_device_id,request_generation,
     request_protection_epoch,request_protection_revision,request_hash,
     request_hash_scheme,outcome,result_revision,changed_count,created_at)
SELECT account_id,client_mutation_id,source_device_id,request_generation,
       request_protection_epoch,request_protection_revision,request_hash,
       request_hash_scheme,outcome,result_revision,changed_count,created_at
FROM cloud_sync_push_mutations;

CREATE TABLE cloud_sync_push_receipt_results (
    account_id UUID NOT NULL,
    client_mutation_id UUID NOT NULL,
    resource_kind TEXT NOT NULL
        CHECK (resource_kind IN ('host', 'ai_provider_account', 'proxy_profile')),
    resource_id UUID NOT NULL,
    result_revision BIGINT NOT NULL CHECK (result_revision > 0),
    PRIMARY KEY (account_id, client_mutation_id, resource_kind, resource_id),
    FOREIGN KEY (account_id, client_mutation_id)
        REFERENCES cloud_sync_push_receipts(account_id, client_mutation_id)
        ON DELETE CASCADE
);

INSERT INTO cloud_sync_push_receipt_results
    (account_id,client_mutation_id,resource_kind,resource_id,result_revision)
SELECT results.account_id,results.client_mutation_id,results.resource_kind,
       results.resource_id,results.result_revision
FROM cloud_sync_push_results AS results
JOIN cloud_sync_push_receipts AS receipts
  ON receipts.account_id=results.account_id
 AND receipts.client_mutation_id=results.client_mutation_id;

CREATE FUNCTION retain_sync_push_receipt()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $creation_push_receipt$
BEGIN
    INSERT INTO cloud_sync_push_receipts
        (account_id,client_mutation_id,source_device_id,request_generation,
         request_protection_epoch,request_protection_revision,request_hash,
         request_hash_scheme,outcome,result_revision,changed_count,created_at)
    VALUES
        (NEW.account_id,NEW.client_mutation_id,NEW.source_device_id,
         NEW.request_generation,NEW.request_protection_epoch,
         NEW.request_protection_revision,NEW.request_hash,
         NEW.request_hash_scheme,NEW.outcome,NEW.result_revision,
         NEW.changed_count,NEW.created_at);
    RETURN NEW;
END;
$creation_push_receipt$;

CREATE TRIGGER cloud_sync_push_mutations_retain_receipt
AFTER INSERT ON cloud_sync_push_mutations
FOR EACH ROW EXECUTE FUNCTION retain_sync_push_receipt();

CREATE FUNCTION retain_sync_push_receipt_result()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $creation_push_receipt_result$
BEGIN
    IF EXISTS (
        SELECT 1 FROM cloud_sync_push_receipts
        WHERE account_id=NEW.account_id
          AND client_mutation_id=NEW.client_mutation_id
    ) THEN
        INSERT INTO cloud_sync_push_receipt_results
            (account_id,client_mutation_id,resource_kind,resource_id,result_revision)
        VALUES
            (NEW.account_id,NEW.client_mutation_id,NEW.resource_kind,
             NEW.resource_id,NEW.result_revision);
    END IF;
    RETURN NEW;
END;
$creation_push_receipt_result$;

CREATE TRIGGER cloud_sync_push_results_retain_receipt
AFTER INSERT ON cloud_sync_push_results
FOR EACH ROW EXECUTE FUNCTION retain_sync_push_receipt_result();

CREATE FUNCTION reject_legacy_host_metadata_write()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $creation_host_metadata_guard$
BEGIN
    IF NOT NEW.metadata_encrypted THEN
        RAISE EXCEPTION USING
            ERRCODE = '23514',
            MESSAGE = 'legacy plaintext Host metadata writes require sync contract upgrade';
    END IF;
    RETURN NEW;
END;
$creation_host_metadata_guard$;

CREATE TRIGGER cloud_hosts_reject_legacy_metadata_write
BEFORE INSERT OR UPDATE ON cloud_hosts
FOR EACH ROW EXECUTE FUNCTION reject_legacy_host_metadata_write();

CREATE TRIGGER cloud_host_versions_reject_legacy_metadata_write
BEFORE INSERT OR UPDATE ON cloud_host_versions
FOR EACH ROW EXECUTE FUNCTION reject_legacy_host_metadata_write();
