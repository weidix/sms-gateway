use std::{future::Future, io};

use anyhow::{bail, Context, Result};
use sqlx::SqlitePool;

use crate::{
    db::inbox::{self, ArchivedMessages},
    decode::{decode_stored_messages, received_pdus, StoredPdu},
    modem::{SmsStorageStatus, SmsType},
};

/// The serialized AT-command interface required by the inbox importer.
pub(super) trait InboxDevice: Sync {
    fn command(&self, command: &str) -> impl Future<Output = io::Result<String>> + Send;
}

/// Archives complete received messages before deleting their exact device records.
/// The caller holds the modem inbox lock for this entire operation.
pub(super) async fn synchronize(
    device: &impl InboxDevice,
    pool: &SqlitePool,
    sim_id: &str,
    sms_type: SmsType,
) -> Result<ArchivedMessages> {
    if sim_id.is_empty() {
        bail!("Cannot archive device messages without a SIM identity");
    }
    let response = device.command("AT+CPMS?\r\n").await?;
    let storage = SmsStorageStatus::from_response(&response)
        .filter(|status| !status.read_storage.is_empty())
        .context("Cannot determine the storage used to read and delete SMS")?
        .read_storage;
    let response = device
        .command(&format!("AT+CMGL={}\r\n", sms_type.to_at_command_pdu()))
        .await?;
    let entries = received_pdus(&response);
    let saved = inbox::receipts(pool, sim_id, &storage).await?;
    let (mut acknowledged, pending): (Vec<_>, Vec<_>) = entries
        .into_iter()
        .partition(|entry| saved.get(&entry.index) == Some(&entry.pdu));
    let messages = decode_stored_messages(pending, sim_id);
    let archived = inbox::archive(pool, &storage, &messages).await?;
    acknowledged.extend(messages.into_iter().flat_map(|message| message.parts));

    // Delete high indexes first for devices that compact their storage records.
    acknowledged.sort_by_key(|entry| std::cmp::Reverse(entry.index));
    for entry in acknowledged {
        if let Err(error) = delete_acknowledged(device, &entry).await {
            log::warn!(
                "SMS cleanup deferred for SIM {} storage {} index {}: {:#}",
                sim_id,
                storage,
                entry.index,
                error
            );
            // After an ambiguous serial failure, rescan before attempting more deletions.
            break;
        }
        if let Err(error) =
            inbox::forget_receipt(pool, sim_id, &storage, entry.index, &entry.pdu).await
        {
            log::warn!(
                "Failed to clear SMS deletion receipt for {}: {}",
                sim_id,
                error
            );
        }
    }
    Ok(archived)
}

async fn delete_acknowledged(device: &impl InboxDevice, entry: &StoredPdu) -> Result<()> {
    // Indexes can be reused after deletion or reconnection. Re-read the exact PDU
    // before each deletion; a stale acknowledgement never authorizes a new SMS.
    let response = device
        .command(&format!("AT+CMGR={}\r\n", entry.index))
        .await?;
    let mut lines = response.lines().map(str::trim);
    let pdu = lines
        .find(|line| line.starts_with("+CMGR:"))
        .and_then(|_| lines.next());
    if !pdu.is_some_and(|pdu| pdu.eq_ignore_ascii_case(&entry.pdu)) {
        bail!(
            "Device SMS changed since it was archived; preserving index {}",
            entry.index
        );
    }
    device
        .command(&format!("AT+CMGD={},0\r\n", entry.index))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests;
