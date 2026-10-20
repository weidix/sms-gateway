use std::collections::HashMap;

use anyhow::Result;
use sqlx::SqlitePool;

use crate::{db::ModemSMS, decode::IndexedSms};

/// Newly archived messages, excluding retries and messages imported by older versions.
#[derive(Default)]
pub(crate) struct ArchivedMessages {
    pub messages: Vec<ModemSMS>,
    pub contact_ids: Vec<String>,
}

/// Loads durable acknowledgements for the selected SIM and physical storage.
pub(crate) async fn receipts(
    pool: &SqlitePool,
    sim_id: &str,
    storage: &str,
) -> Result<HashMap<u32, String>> {
    let rows: Vec<(u32, String)> = sqlx::query_as(
        "SELECT modem_index, pdu FROM modem_sms_receipts WHERE sim_id = ? AND storage = ?",
    )
    .bind(sim_id)
    .bind(storage)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().collect())
}

/// Saves complete messages and their device acknowledgements in one transaction.
/// Existing history is matched by SIM, sender, timestamp, content, and direction.
pub(crate) async fn archive(
    pool: &SqlitePool,
    storage: &str,
    messages: &[IndexedSms],
) -> Result<ArchivedMessages> {
    let mut transaction = pool.begin().await?;
    let mut archived = ArchivedMessages::default();
    for message in messages {
        let sms = &message.sms;
        let contact_id = sms.get_contact_id(&mut transaction).await?;
        let existing: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sms WHERE contact_id = ? AND sim_id = ? \
             AND timestamp = ? AND message = ? AND send = ?)",
        )
        .bind(&contact_id)
        .bind(&sms.sim_id)
        .bind(sms.timestamp)
        .bind(&sms.message)
        .bind(sms.send)
        .fetch_one(&mut *transaction)
        .await?;
        if !existing {
            sms.insert_transaction(&mut transaction).await?;
            archived.messages.push(sms.clone());
            archived.contact_ids.push(contact_id);
        }
        for part in &message.parts {
            sqlx::query(
                "INSERT INTO modem_sms_receipts (sim_id, storage, modem_index, pdu) \
                 VALUES (?, ?, ?, ?) ON CONFLICT(sim_id, storage, modem_index) \
                 DO UPDATE SET pdu = excluded.pdu",
            )
            .bind(&sms.sim_id)
            .bind(storage)
            .bind(part.index)
            .bind(&part.pdu)
            .execute(&mut *transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(archived)
}

/// Removes only the acknowledgement for the exact PDU confirmed as deleted.
pub(crate) async fn forget_receipt(
    pool: &SqlitePool,
    sim_id: &str,
    storage: &str,
    index: u32,
    pdu: &str,
) -> Result<()> {
    sqlx::query(
        "DELETE FROM modem_sms_receipts \
         WHERE sim_id = ? AND storage = ? AND modem_index = ? AND pdu = ?",
    )
    .bind(sim_id)
    .bind(storage)
    .bind(index)
    .bind(pdu)
    .execute(pool)
    .await?;
    Ok(())
}
