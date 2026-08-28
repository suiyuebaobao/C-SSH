async fn replace_envelope(
    tx: &mut DbTransaction<'_>,
    actor: DeviceActor,
    generation: i64,
    epoch: i64,
    revision: i64,
    envelope: &ValidatedEnvelope,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE cloud_data_protection_envelopes
         SET sync_generation=$2, protection_epoch=$3, protection_revision=$4,
             format_version=1, kdf_algorithm='argon2id', kdf_version=19,
             kdf_memory_kib=19456, kdf_iterations=2, kdf_parallelism=1,
             kdf_output_length=32, salt=$5, nonce=$6, wrapped_data_key=$7,
             source_device_id=$8, updated_at=now()
         WHERE account_id=$1",
    )
    .bind(actor.account_id())
    .bind(generation)
    .bind(epoch)
    .bind(revision)
    .bind(&envelope.salt)
    .bind(&envelope.nonce)
    .bind(&envelope.wrapped_data_key)
    .bind(actor.device_id())
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

pub(super) async fn update_state(
    tx: &mut DbTransaction<'_>,
    account_id: Uuid,
    generation: i64,
    epoch: i64,
    revision: i64,
    current_revision: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE cloud_host_sync_states
         SET sync_generation=$2, protection_epoch=$3, protection_revision=$4,
             current_revision=$5, updated_at=now()
         WHERE account_id=$1",
    )
    .bind(account_id)
    .bind(generation)
    .bind(epoch)
    .bind(revision)
    .bind(current_revision)
    .execute(&mut **tx)
    .await
    .map_err(storage)?;
    Ok(())
}

async fn require_envelope_absent(tx: &mut DbTransaction<'_>, account_id: Uuid) -> AppResult<()> {
    let present = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM cloud_data_protection_envelopes WHERE account_id=$1)",
    )
    .bind(account_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(storage)?;
    if present {
        Err(AppError::Conflict(
            "account data protection is already configured".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn checked_next(value: i64, field: &str) -> AppResult<i64> {
    value
        .checked_add(1)
        .ok_or_else(|| AppError::Conflict(format!("{field} cannot advance")))
}
