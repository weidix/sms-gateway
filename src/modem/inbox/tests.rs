use std::collections::BTreeMap;

use sqlx::sqlite::SqlitePoolOptions;
use tokio::sync::Mutex;

use super::*;

const SINGLE: &str = "0000049121430008620102214365000400480069";
const FIRST: &str = "0040049121430008620102214365000A0500032A020100480069";
const SECOND: &str = "0040049121430008620102214365000A0500032A020200210021";

struct DeviceState {
    entries: BTreeMap<u32, String>,
    commands: Vec<String>,
    fail_delete: Option<u32>,
    lose_delete_response: bool,
    replace_on_read: bool,
}

struct Device {
    state: Mutex<DeviceState>,
    pool: SqlitePool,
    storage: &'static str,
}

impl Device {
    async fn new(entries: &[(u32, &str)]) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        Self {
            state: Mutex::new(DeviceState {
                entries: entries
                    .iter()
                    .map(|(index, pdu)| (*index, pdu.to_string()))
                    .collect(),
                commands: Vec::new(),
                fail_delete: None,
                lose_delete_response: false,
                replace_on_read: false,
            }),
            pool,
            storage: "SM",
        }
    }

    async fn sync(&self) -> Result<ArchivedMessages> {
        synchronize(self, &self.pool, "sim", SmsType::All).await
    }

    async fn count(&self, table: &str) -> i64 {
        sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}

impl InboxDevice for Device {
    async fn command(&self, command: &str) -> io::Result<String> {
        let mut state = self.state.lock().await;
        state.commands.push(command.to_string());
        let command = command.trim();
        if command == "AT+CPMS?" {
            return Ok(format!(
                "+CPMS: \"{0}\",2,40,\"{0}\",2,40,\"{0}\",2,40\r\nOK\r\n",
                self.storage
            ));
        }
        if command == "AT+CMGL=4" {
            return Ok(state
                .entries
                .iter()
                .map(|(index, pdu)| format!("+CMGL: {index},1,,20\r\n{pdu}\r\n"))
                .collect::<String>()
                + "OK\r\n");
        }
        if let Some(index) = command.strip_prefix("AT+CMGR=") {
            let index: u32 = index.parse().unwrap();
            if state.replace_on_read {
                state
                    .entries
                    .insert(index, SINGLE.replace("00480069", "00420079"));
                state.replace_on_read = false;
            }
            let pdu = state
                .entries
                .get(&index)
                .ok_or_else(|| io::Error::other("Missing index"))?;
            return Ok(format!("+CMGR: 1,,20\r\n{pdu}\r\nOK\r\n"));
        }
        if let Some(index) = command.strip_prefix("AT+CMGD=") {
            let index: u32 = index.strip_suffix(",0").unwrap().parse().unwrap();
            // These reads use a separate connection checkout: the message and its
            // acknowledgement must already be committed when deletion is issued.
            assert!(self.count("sms").await > 0);
            let saved = inbox::receipts(&self.pool, "sim", self.storage)
                .await
                .unwrap();
            assert_eq!(saved.get(&index), state.entries.get(&index));
            if state.fail_delete == Some(index) {
                return Err(io::Error::other("Delete failed"));
            }
            state.entries.remove(&index);
            if state.lose_delete_response {
                state.lose_delete_response = false;
                return Err(io::Error::new(io::ErrorKind::TimedOut, "Response lost"));
            }
            return Ok("OK\r\n".to_string());
        }
        panic!("Unexpected command: {command}");
    }
}

#[tokio::test]
async fn commits_archive_before_deleting_and_preserves_web_history() {
    let device = Device::new(&[(1, SINGLE)]).await;
    let archived = device.sync().await.unwrap();
    assert_eq!(archived.messages.len(), 1);
    assert_eq!(archived.messages[0].message, "Hi");
    assert_eq!(device.count("sms").await, 1);
    assert_eq!(device.count("modem_sms_receipts").await, 0);
    assert!(device.state.lock().await.entries.is_empty());
    assert!(device.sync().await.unwrap().messages.is_empty());
}

#[tokio::test]
async fn failed_database_commit_never_authorizes_deletion() {
    let device = Device::new(&[(1, SINGLE)]).await;
    sqlx::query("CREATE TRIGGER fail_receipt BEFORE INSERT ON modem_sms_receipts BEGIN SELECT RAISE(ABORT, 'disk full'); END")
        .execute(&device.pool).await.unwrap();
    assert!(device.sync().await.is_err());
    assert_eq!(device.count("sms").await, 0);
    assert_eq!(device.count("contacts").await, 0);
    assert_eq!(device.count("modem_sms_receipts").await, 0);
    assert!(device
        .state
        .lock()
        .await
        .commands
        .iter()
        .all(|command| !command.starts_with("AT+CMGD")));
    sqlx::query("DROP TRIGGER fail_receipt")
        .execute(&device.pool)
        .await
        .unwrap();
    // Device messages are already read; the all-message scan still retries them.
    assert_eq!(device.sync().await.unwrap().messages.len(), 1);
    assert!(device.state.lock().await.entries.is_empty());
}

#[tokio::test]
async fn deletion_failure_retries_without_duplicate_history_or_notifications() {
    let device = Device::new(&[(1, SINGLE)]).await;
    device.state.lock().await.fail_delete = Some(1);
    assert_eq!(device.sync().await.unwrap().messages.len(), 1);
    assert_eq!(device.count("modem_sms_receipts").await, 1);
    assert!(device.sync().await.unwrap().messages.is_empty());
    device.state.lock().await.fail_delete = None;
    assert!(device.sync().await.unwrap().messages.is_empty());
    assert_eq!(device.count("sms").await, 1);
    assert!(device.state.lock().await.entries.is_empty());
}

#[tokio::test]
async fn multipart_cleanup_resumes_from_durable_receipts_after_partial_deletion() {
    let device = Device::new(&[(1, FIRST), (2, SECOND)]).await;
    device.state.lock().await.fail_delete = Some(1);
    let first = device.sync().await.unwrap();
    assert_eq!(first.messages[0].message, "Hi!!");
    assert_eq!(device.state.lock().await.entries.len(), 1);
    assert_eq!(device.count("modem_sms_receipts").await, 1);
    // synchronize has no in-memory receipt state between calls, like a restart.
    device.state.lock().await.fail_delete = None;
    assert!(device.sync().await.unwrap().messages.is_empty());
    assert!(device.state.lock().await.entries.is_empty());
    assert_eq!(device.count("sms").await, 1);
}

#[tokio::test]
async fn lost_delete_response_does_not_delete_a_new_message_reusing_the_index() {
    let device = Device::new(&[(1, SINGLE)]).await;
    device.state.lock().await.lose_delete_response = true;
    assert_eq!(device.sync().await.unwrap().messages.len(), 1);
    assert!(device.state.lock().await.entries.is_empty());
    device
        .state
        .lock()
        .await
        .entries
        .insert(1, SINGLE.replace("00480069", "00420079"));
    let next = device.sync().await.unwrap();
    assert_eq!(next.messages[0].message, "By");
    assert_eq!(device.count("sms").await, 2);
}

#[tokio::test]
async fn changed_pdu_is_preserved_until_it_has_been_archived() {
    let device = Device::new(&[(1, SINGLE)]).await;
    device.state.lock().await.replace_on_read = true;
    assert_eq!(device.sync().await.unwrap().messages.len(), 1);
    assert_eq!(device.state.lock().await.entries.len(), 1);
    assert!(device
        .state
        .lock()
        .await
        .commands
        .iter()
        .all(|command| !command.starts_with("AT+CMGD")));
    assert_eq!(device.sync().await.unwrap().messages[0].message, "By");
    assert_eq!(device.count("sms").await, 2);
}

#[tokio::test]
async fn incomplete_and_malformed_messages_stay_on_the_device() {
    let device = Device::new(&[(1, FIRST), (3, "00"), (4, "NOTHEX")]).await;
    assert!(device.sync().await.unwrap().messages.is_empty());
    assert_eq!(device.state.lock().await.entries.len(), 3);
    assert_eq!(device.count("sms").await, 0);
    device
        .state
        .lock()
        .await
        .entries
        .insert(2, SECOND.to_string());
    assert_eq!(device.sync().await.unwrap().messages[0].message, "Hi!!");
    assert_eq!(device.state.lock().await.entries.len(), 2);
}

#[tokio::test]
async fn existing_history_is_not_duplicated_during_upgrade() {
    let device = Device::new(&[(1, SINGLE)]).await;
    let messages = decode_stored_messages(
        vec![StoredPdu {
            index: 1,
            pdu: SINGLE.to_string(),
        }],
        "sim",
    );
    let mut transaction = device.pool.begin().await.unwrap();
    messages[0]
        .sms
        .insert_transaction(&mut transaction)
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    assert!(device.sync().await.unwrap().messages.is_empty());
    assert_eq!(device.count("sms").await, 1);
    assert!(device.state.lock().await.entries.is_empty());
}

#[tokio::test]
async fn receipts_are_isolated_by_sim_and_storage() {
    let mut device = Device::new(&[(1, FIRST), (2, SECOND)]).await;
    device.state.lock().await.fail_delete = Some(2);
    device.sync().await.unwrap();
    device.storage = "ME";
    device.state.lock().await.entries.remove(&2);
    // The SM acknowledgement must not permit deleting an incomplete ME message.
    assert!(device.sync().await.unwrap().messages.is_empty());
    assert_eq!(device.state.lock().await.entries.len(), 1);
    assert!(inbox::receipts(&device.pool, "other-sim", "SM")
        .await
        .unwrap()
        .is_empty());
}
