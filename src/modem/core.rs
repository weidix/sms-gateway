use chrono::{Local, Timelike};
use futures::stream::{FuturesUnordered, StreamExt};
use log::{debug, error, info};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::api::SseManager;
use crate::config::SmsStorage;
use crate::db::{Contact, ModemSMS, SimCard, Sms};
use crate::decode::parse_pdu_sms;
use crate::webhook;

use super::inbox::{self, InboxDevice};
use super::pdu::build_pdu;
use super::types::*;

const TERMINATORS: &[&[u8]] = &[
    b"\r\nOK\r\n",
    b"\r\nERROR\r\n",
    b"\r\n> ",
    b"\r\n+CME ERROR",
    b"\r\n+CMS ERROR",
];

const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_CONCURRENT_COMMANDS: usize = 5;

struct SerialConnection {
    stream: Option<SerialStream>,
    state: ConnectionState,
    sms_storage: Option<SmsStorage>,
}

impl SerialConnection {
    fn disconnect(&mut self) {
        // Dropping the stream releases the old device even if its path was replaced.
        self.stream = None;
        self.state = ConnectionState::Disconnected;
    }
}

pub struct Modem {
    pub name: String,
    pub com_port: String,
    pub baud_rate: u32,
    command_tx: mpsc::UnboundedSender<ATCommand>,
    serial_connection: Arc<Mutex<SerialConnection>>,
    pub sim_id: RwLock<Option<String>>,
    inbox_lock: Mutex<()>,
}

impl Modem {
    fn init_commands() -> [(&'static str, &'static str); 4] {
        [
            ("ATE0\r\n", "Disable echo"),
            ("AT+CMEE=1\r\n", "Enable error messages"),
            ("AT+CMGF=0\r\n", "Set PDU mode"),
            ("AT+CSCS=\"UCS2\"\r\n", "Set character encoding"),
        ]
    }

    fn sms_storage_code(storage: SmsStorage) -> &'static str {
        match storage {
            SmsStorage::SIM => "SM",
            SmsStorage::ME => "ME",
            SmsStorage::MT => "MT",
        }
    }

    pub async fn new(com_port: &str, baud_rate: u32, name: &str) -> io::Result<Self> {
        let serial_stream = Self::create_serial_connection(com_port, baud_rate).await?;

        let (command_tx, command_rx) = mpsc::unbounded_channel::<ATCommand>();
        let serial_connection = Arc::new(Mutex::new(SerialConnection {
            stream: Some(serial_stream),
            state: ConnectionState::Connected,
            sms_storage: None,
        }));
        let command_semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_COMMANDS));

        let name_clone = name.to_string();
        let com_port_clone = com_port.to_string();
        let serial_connection_clone = serial_connection.clone();
        let semaphore_clone = command_semaphore.clone();

        tokio::spawn(async move {
            Self::command_processor(
                command_rx,
                serial_connection_clone,
                &name_clone,
                &com_port_clone,
                baud_rate,
                semaphore_clone,
            )
            .await;
        });

        info!("device:{}, com:{} connected successfully", name, com_port);

        Ok(Modem {
            name: name.to_string(),
            com_port: com_port.to_string(),
            baud_rate,
            command_tx,
            serial_connection,
            sim_id: RwLock::new(None),
            inbox_lock: Mutex::new(()),
        })
    }

    async fn create_serial_connection(
        com_port: &str,
        baud_rate: u32,
    ) -> tokio_serial::Result<SerialStream> {
        tokio_serial::new(com_port, baud_rate)
            .timeout(Duration::from_secs(10))
            .flow_control(tokio_serial::FlowControl::None)
            .open_native_async()
    }

    async fn command_processor(
        mut command_rx: mpsc::UnboundedReceiver<ATCommand>,
        serial_connection: Arc<Mutex<SerialConnection>>,
        name: &str,
        com_port: &str,
        baud_rate: u32,
        semaphore: Arc<Semaphore>,
    ) {
        let mut futures = FuturesUnordered::new();

        loop {
            tokio::select! {
                Some(at_command) = command_rx.recv() => {
                    let serial_connection = serial_connection.clone();
                    let name = name.to_string();
                    let com_port = com_port.to_string();
                    let semaphore = semaphore.clone();

                    futures.push(tokio::spawn(async move {
                        let _permit = semaphore.acquire().await;
                        Self::execute_command_with_retry(
                            serial_connection,
                            at_command,
                            &name,
                            &com_port,
                            baud_rate,
                        ).await
                    }));
                }
                Some(result) = futures.next() => {
                    if let Err(e) = result {
                        error!("Command execution task failed: {}", e);
                    }
                }
                else => break,
            }
        }
    }

    async fn execute_command_with_retry(
        serial_connection: Arc<Mutex<SerialConnection>>,
        mut at_command: ATCommand,
        name: &str,
        com_port: &str,
        baud_rate: u32,
    ) {
        while at_command.retries < MAX_RETRIES {
            // Opening, initialization, and commands share one lock so no command
            // can use a stale stream or an incompletely initialized replacement.
            let mut connection = serial_connection.lock().await;
            if connection.stream.is_none() {
                if let Err(err) =
                    Self::attempt_reconnection(&mut connection, com_port, baud_rate, name).await
                {
                    at_command.retries += 1;
                    drop(connection);
                    if at_command.retries < MAX_RETRIES {
                        tokio::time::sleep(RETRY_DELAY).await;
                        continue;
                    }
                    let _ = at_command.response_tx.send(Err(err));
                    return;
                }
            }

            let result = match connection.stream.as_mut() {
                Some(serial) => {
                    Self::execute_single_command(serial, &at_command.command, name).await
                }
                None => Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "Serial port not connected",
                )),
            };
            if let Err(err) = &result {
                connection.disconnect();
                error!(
                    "Serial connection lost for {} on {}: {}",
                    name, com_port, err
                );
            }
            // A response may be lost after an SMS was sent or deleted. Never
            // replay a command that reached the transport; recover on the next one.
            let _ = at_command.response_tx.send(result);
            return;
        }

        let _ = at_command
            .response_tx
            .send(Err(io::Error::other("Maximum retries exceeded")));
    }

    async fn attempt_reconnection(
        connection: &mut SerialConnection,
        com_port: &str,
        baud_rate: u32,
        name: &str,
    ) -> io::Result<()> {
        connection.disconnect();
        connection.state = ConnectionState::Reconnecting;
        info!("Attempting to reconnect to {} on {}", name, com_port);

        let result = async {
            let mut stream = Self::create_serial_connection(com_port, baud_rate).await?;
            Self::initialize_serial(&mut stream, connection.sms_storage, name).await?;
            Ok::<_, io::Error>(stream)
        }
        .await;

        match result {
            Ok(stream) => {
                connection.stream = Some(stream);
                connection.state = ConnectionState::Connected;
                info!(
                    "Successfully reconnected and initialized {} on {}",
                    name, com_port
                );
                Ok(())
            }
            Err(err) => {
                connection.disconnect();
                error!("Reconnection failed for {} on {}: {}", name, com_port, err);
                Err(err)
            }
        }
    }

    async fn initialize_serial(
        stream: &mut SerialStream,
        sms_storage: Option<SmsStorage>,
        name: &str,
    ) -> io::Result<()> {
        for (command, description) in Self::init_commands() {
            if let Err(err) = Self::execute_serial_with_ok(stream, command, name).await {
                error!("Failed to {} for {}: {}", description, name, err);
                return Err(err);
            }
        }
        if let Some(storage) = sms_storage {
            let command = Self::sms_storage_command(storage);
            Self::execute_serial_with_ok(stream, &command, name).await?;
        }
        Ok(())
    }

    fn sms_storage_command(storage: SmsStorage) -> String {
        let storage = Self::sms_storage_code(storage);
        format!("AT+CPMS=\"{0}\",\"{0}\",\"{0}\"\r\n", storage)
    }

    async fn execute_serial_with_ok(
        stream: &mut SerialStream,
        command: &str,
        name: &str,
    ) -> io::Result<String> {
        let response = Self::execute_single_command(stream, command, name).await?;
        Self::validate_ok_response(response)
    }

    async fn execute_single_command(
        serial_stream: &mut SerialStream,
        command: &str,
        name: &str,
    ) -> io::Result<String> {
        debug!("TX [{}]: {}", name, Self::format_log(command));
        tokio::time::timeout(Duration::from_secs(30), async {
            // SerialStream writes directly to the device. Its flush can block
            // in tcdrain, preventing the timeout from detecting a lost device.
            serial_stream.write_all(command.as_bytes()).await?;
            Self::read_response_buffered(serial_stream, name).await
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Command timeout for device {}", name),
            )
        })?
    }

    async fn read_response_buffered(
        serial_stream: &mut (impl AsyncRead + Unpin),
        name: &str,
    ) -> io::Result<String> {
        let mut buffer = Vec::with_capacity(4096);
        let mut temp_buf = [0u8; 1024];

        loop {
            let bytes_read = serial_stream.read(&mut temp_buf).await?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("Serial connection closed for device {}", name),
                ));
            }
            buffer.extend_from_slice(&temp_buf[..bytes_read]);
            if let Some((terminator, pos)) = Self::find_terminator(&buffer) {
                let end_pos = pos + terminator.len();
                let response = String::from_utf8_lossy(&buffer[..end_pos]).into_owned();
                debug!("RX [{}]: {}", name, Self::format_log(&response));
                return Ok(response);
            }
        }
    }

    fn find_terminator(buffer: &[u8]) -> Option<(&'static [u8], usize)> {
        TERMINATORS
            .iter()
            .filter_map(|&t| {
                buffer
                    .windows(t.len())
                    .position(|w| w == t)
                    .map(|pos| (t, pos))
            })
            .max_by_key(|&(_, pos)| pos)
    }

    pub async fn init_modem(&mut self, sms_storage: Option<SmsStorage>) -> io::Result<()> {
        {
            let mut connection = self.serial_connection.lock().await;
            connection.sms_storage = sms_storage;
        }
        for (cmd, description) in Self::init_commands() {
            if let Err(err) = self.send_command_with_ok(cmd).await {
                error!("Failed to {}: {}", description, err);
                return Err(err);
            }
        }
        if let Some(storage) = sms_storage {
            self.configure_sms_storage(storage).await?;
        }
        if let Err(err) = self.refresh_sim_info().await {
            log::warn!(
                "Failed to initialize SIM info for device {}: {}",
                self.name,
                err
            );
        }
        Ok(())
    }

    async fn configure_sms_storage(&self, storage: SmsStorage) -> io::Result<()> {
        let mut connection = self.serial_connection.lock().await;
        if connection.stream.is_none() {
            Self::attempt_reconnection(&mut connection, &self.com_port, self.baud_rate, &self.name)
                .await?;
        }
        let result = match connection.stream.as_mut() {
            Some(stream) => {
                Self::execute_serial_with_ok(
                    stream,
                    &Self::sms_storage_command(storage),
                    &self.name,
                )
                .await
            }
            None => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Serial port not connected",
            )),
        };
        match result {
            Ok(_) => {
                connection.sms_storage = Some(storage);
                info!(
                    "SMS storage set to {} for device {}",
                    Self::sms_storage_code(storage),
                    self.name
                );
                Ok(())
            }
            Err(err) => {
                connection.disconnect();
                Err(err)
            }
        }
    }

    async fn refresh_sim_info(&self) -> anyhow::Result<()> {
        let (iccid_result, imsi_result, phone_result) = tokio::join!(
            self.get_sim_iccid(),
            self.get_sim_imsi(),
            self.get_phone_number()
        );

        let iccid = iccid_result.ok().flatten();
        let imsi = imsi_result.ok().flatten();
        let phone_number = phone_result.ok().flatten();

        if let Some(iccid) = iccid {
            match SimCard::find_or_create_with_phone(&iccid, imsi, phone_number).await {
                Ok(_) => {
                    *self.sim_id.write().await = Some(iccid.clone());
                    info!(
                        "SIM card initialized for device {}: ICCID={}",
                        self.name, iccid
                    );
                }
                Err(e) => {
                    error!("Failed to store SIM card info: {}", e);
                    return Err(e);
                }
            }
        } else {
            log::warn!("Could not retrieve SIM card ICCID for device {}", self.name);
        }

        Ok(())
    }

    pub async fn probe_at(&self) -> io::Result<()> {
        self.send_command_with_ok("AT\r\n").await.map(|_| ())
    }

    pub async fn execute_at_command(&self, command: &str) -> io::Result<String> {
        let _inbox_guard = self.inbox_lock.lock().await;
        let command = Self::normalize_at_command(command);
        self.send_command(&command).await
    }

    /// Close the current serial handle, reopen the configured path, and restore
    /// module settings before allowing further commands to use the connection.
    pub async fn reinitialize_runtime(&self, sms_storage: Option<SmsStorage>) -> io::Result<()> {
        let _inbox_guard = self.inbox_lock.lock().await;
        {
            let mut connection = self.serial_connection.lock().await;
            connection.sms_storage = sms_storage;
            connection.disconnect();
            for attempt in 1..=MAX_RETRIES {
                match Self::attempt_reconnection(
                    &mut connection,
                    &self.com_port,
                    self.baud_rate,
                    &self.name,
                )
                .await
                {
                    Ok(()) => break,
                    Err(err) if attempt == MAX_RETRIES => return Err(err),
                    Err(_) => tokio::time::sleep(RETRY_DELAY).await,
                }
            }
        }
        if let Err(err) = self.refresh_sim_info().await {
            log::warn!(
                "Failed to refresh SIM info during runtime reinitialization for device {}: {}",
                self.name,
                err
            );
        }
        Ok(())
    }

    pub async fn soft_restart(&self) -> io::Result<()> {
        let _inbox_guard = self.inbox_lock.lock().await;
        Self::validate_soft_restart_result(self.send_command("AT+CFUN=1,1\r\n").await)
    }

    async fn send_command(&self, command: &str) -> io::Result<String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        let at_command = ATCommand {
            command: command.to_string(),
            response_tx,
            // Deletion responses can be lost after success. Re-read the slot on
            // the next scan instead of blindly retrying a destructive command.
            retries: if command.starts_with("AT+CMGD=") {
                MAX_RETRIES - 1
            } else {
                0
            },
        };

        self.command_tx
            .send(at_command)
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Command queue closed"))?;

        response_rx
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Response channel closed"))?
    }

    fn normalize_at_command(command: &str) -> String {
        let command = command.trim_end_matches(['\r', '\n']);
        format!("{command}\r\n")
    }

    fn validate_soft_restart_result(result: io::Result<String>) -> io::Result<()> {
        match result {
            Ok(response) => {
                if Self::response_has_error(&response) {
                    error!("Soft restart failed: {}", response);
                    Err(io::Error::other(format!(
                        "Command failed: {}",
                        Self::format_log(&response)
                    )))
                } else {
                    Ok(())
                }
            }
            Err(err) if Self::is_expected_restart_disconnect(&err) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn response_has_error(response: &str) -> bool {
        response.contains("\r\nERROR\r\n")
            || response.contains("+CME ERROR")
            || response.contains("+CMS ERROR")
    }

    fn is_expected_restart_disconnect(err: &io::Error) -> bool {
        matches!(
            err.kind(),
            io::ErrorKind::BrokenPipe
                | io::ErrorKind::NotConnected
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::UnexpectedEof
        )
    }

    async fn send_command_with_ok(&self, command: &str) -> io::Result<String> {
        let response = self.send_command(command).await?;
        Self::validate_ok_response(response)
    }

    fn validate_ok_response(response: String) -> io::Result<String> {
        if response.contains("OK\r\n") {
            Ok(response)
        } else {
            error!("Command failed: {}", response);
            Err(io::Error::other(format!(
                "Command failed: {}",
                Self::format_log(&response)
            )))
        }
    }

    async fn send_sms_content<F>(
        &self,
        setup_cmd: &str,
        message: &str,
        transform_fn: F,
    ) -> anyhow::Result<String>
    where
        F: FnOnce(&str) -> anyhow::Result<String>,
    {
        let prompt_response = self.send_command(setup_cmd).await?;

        if !prompt_response.contains("> ") {
            return Err(anyhow::anyhow!("SMS prompt not received"));
        }

        let transformed_message = transform_fn(message)?;
        let full_message = format!("{}\x1A", transformed_message);

        let final_response = self.send_command(&full_message).await?;

        if final_response.contains("OK\r\n") && final_response.contains("+CMGS:") {
            Ok(final_response)
        } else {
            error!(
                "Incomplete SMS response: {}",
                Self::format_log(&final_response)
            );
            Err(anyhow::anyhow!("Incomplete SMS response"))
        }
    }

    pub async fn send_sms_pdu(
        &self,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        info!("Sending SMS via PDU to {}: {}", contact.name, message);

        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();

        let sms = Sms {
            id: 0,
            contact_id: contact.id.clone(),
            timestamp: Local::now().naive_local().with_nanosecond(0).unwrap(),
            message: message.to_string(),
            sim_id,
            send: true,
            status: crate::db::SmsStatus::Loading,
        };

        let sms_id = sms.insert().await?;

        match self.send_pdu_message(&contact.name, message).await {
            Ok(_) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Read).await?;
                Ok((sms_id, contact.id.clone()))
            }
            Err(e) => {
                Sms::update_status_by_id(sms_id, crate::db::SmsStatus::Failed).await?;
                Err(e)
            }
        }
    }

    async fn send_pdu_message(&self, phone: &str, message: &str) -> anyhow::Result<()> {
        let (pdu_data, tpdu_length) = build_pdu(phone, message)?;

        self.send_sms_content(&format!("AT+CMGS={}\r", tpdu_length), &pdu_data, |pdu| {
            Ok(pdu.to_string())
        })
        .await?;

        Ok(())
    }

    /// Archives received SMS, cleans up device copies, and notifies newly saved conversations.
    pub async fn read_sms_async_insert(
        &self,
        sms_type: SmsType,
        sse_manager: Arc<SseManager>,
        webhook_manager: Option<webhook::WebhookManager>,
    ) -> anyhow::Result<()> {
        let archived = self.archive_device_sms(sms_type).await?;
        if let Some(webhook_manager) = webhook_manager {
            for sms in archived.messages {
                if let Err(error) = webhook_manager.send(sms) {
                    log::error!("Failed to enqueue archived SMS webhook: {}", error);
                }
            }
        }
        if !archived.contact_ids.is_empty() {
            match crate::db::Conversation::query_by_contact_ids(&archived.contact_ids).await {
                Ok(conversations) => sse_manager.send(conversations),
                Err(error) => log::error!("Failed to load archived SMS conversations: {}", error),
            }
        }
        Ok(())
    }

    async fn archive_device_sms(
        &self,
        sms_type: SmsType,
    ) -> anyhow::Result<crate::db::inbox::ArchivedMessages> {
        let _inbox_guard = self.inbox_lock.lock().await;
        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();
        inbox::synchronize(self, crate::db::get_pool()?, &sim_id, sms_type).await
    }

    pub async fn read_sms(&self, sms_type: SmsType) -> io::Result<Vec<ModemSMS>> {
        let _inbox_guard = self.inbox_lock.lock().await;
        let command = format!("AT+CMGL={}\r\n", sms_type.to_at_command_pdu());
        let response = self.send_command_with_ok(&command).await?;

        let sim_id = self.sim_id.read().await.clone().unwrap_or_default();
        Ok(parse_pdu_sms(&response, &sim_id))
    }

    async fn get_modem_info<T>(
        &self,
        command: &str,
        parser: fn(&str) -> Option<T>,
    ) -> io::Result<Option<T>> {
        let raw_response = self.send_command_with_ok(command).await?;
        let cleaned_response = raw_response.trim().replace("OK", "");
        Ok(parser(&cleaned_response))
    }

    pub async fn get_signal_quality(&self) -> io::Result<Option<SignalQuality>> {
        self.get_modem_info("AT+CSQ\r\n", SignalQuality::from_response)
            .await
    }

    pub async fn check_network_registration(
        &self,
    ) -> io::Result<Option<NetworkRegistrationStatus>> {
        self.get_modem_info("AT+CREG?\r\n", NetworkRegistrationStatus::from_response)
            .await
    }

    pub async fn check_operator(&self) -> io::Result<Option<OperatorInfo>> {
        self.get_modem_info("AT+COPS?\r\n", OperatorInfo::from_response)
            .await
    }

    pub async fn get_modem_model(&self) -> io::Result<Option<ModemInfo>> {
        self.get_modem_info("AT+CGMM\r\n", ModemInfo::from_response)
            .await
    }

    pub async fn get_sim_iccid(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CCID\r\n", |response| {
            response
                .lines()
                .find(|line| {
                    line.contains("CCID:")
                        || (line.trim().len() >= 19
                            && line.trim().chars().all(|c| c.is_ascii_hexdigit()))
                })
                .map(|line| {
                    if line.contains("CCID:") {
                        line.split("CCID:")
                            .nth(1)
                            .map(|s| s.trim().to_string())
                            .unwrap_or_else(|| line.trim().to_string())
                    } else {
                        line.trim().to_string()
                    }
                })
        })
        .await
    }

    pub async fn get_sim_imsi(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CIMI\r\n", |response| {
            response
                .lines()
                .find(|line| {
                    let trimmed = line.trim();
                    trimmed.len() >= 15 && trimmed.chars().all(|c| c.is_ascii_digit())
                })
                .map(|line| line.trim().to_string())
        })
        .await
    }

    pub async fn get_phone_number(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CNUM\r\n", Self::parse_phone_number)
            .await
    }

    fn parse_phone_number(response: &str) -> Option<String> {
        response
            .lines()
            .find(|line| line.starts_with("+CNUM:"))
            .and_then(|line| {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    parts[1].trim().trim_matches('"').to_string().into()
                } else {
                    None
                }
            })
    }

    fn format_log(input: &str) -> String {
        input
            .replace("\r\n", "\\r\\n")
            .replace("\r", "\\r")
            .replace("\n", "\\n")
    }

    /// Archives received SMS before deleting their device copies, without notifications.
    pub async fn read_sms_sync_insert(&self, sms_type: SmsType) -> anyhow::Result<()> {
        self.archive_device_sms(sms_type).await?;
        Ok(())
    }

    pub async fn get_sms_center(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CSCA?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CSCA:"))
                .and_then(|line| {
                    line.split('"')
                        .nth(1)
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
        })
        .await
    }

    pub async fn get_network_info(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPSI?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPSI:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn get_sim_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPIN?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPIN:"))
                .and_then(|line| line.split(':').nth(1).map(|s| s.trim().to_string()))
        })
        .await
    }

    pub async fn get_memory_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPMS?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPMS:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn get_temperature_info(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+QTEMP?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+QTEMP:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn set_sms_storage(&self, sms_storage: SmsStorage) -> io::Result<()> {
        let _inbox_guard = self.inbox_lock.lock().await;
        self.configure_sms_storage(sms_storage).await
    }

    pub async fn get_sms_storage_status(&self) -> io::Result<Option<String>> {
        self.get_modem_info("AT+CPMS?\r\n", |response| {
            response
                .lines()
                .find(|line| line.starts_with("+CPMS:"))
                .map(|line| line.to_string())
        })
        .await
    }

    pub async fn get_sms_storage_overview(&self) -> io::Result<Option<SmsStorageStatus>> {
        self.get_modem_info("AT+CPMS?\r\n", SmsStorageStatus::from_response)
            .await
    }
}

impl InboxDevice for Modem {
    fn command(
        &self,
        command: &str,
    ) -> impl std::future::Future<Output = io::Result<String>> + Send {
        self.send_command_with_ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::Modem;
    use std::io;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn eof_is_an_error_even_after_a_partial_response() {
        for partial in [b"".as_slice(), b"\r\n+CMGL: 1,0,,20\r\n001122"] {
            let (mut reader, mut writer) = tokio::io::duplex(64);
            writer.write_all(partial).await.unwrap();
            drop(writer);
            let err = Modem::read_response_buffered(&mut reader, "eof-test")
                .await
                .unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::UnexpectedEof);
        }
    }

    #[tokio::test]
    async fn complete_response_still_succeeds_before_eof() {
        let (mut reader, mut writer) = tokio::io::duplex(64);
        writer.write_all(b"\r\nOK\r\n").await.unwrap();
        drop(writer);
        assert_eq!(
            Modem::read_response_buffered(&mut reader, "response-test")
                .await
                .unwrap(),
            "\r\nOK\r\n"
        );
    }

    #[test]
    fn soft_restart_accepts_expected_disconnect_errors() {
        let result =
            Modem::validate_soft_restart_result(Err(io::Error::from(io::ErrorKind::BrokenPipe)));

        assert!(result.is_ok());
    }

    #[test]
    fn soft_restart_rejects_modem_error_responses() {
        let result = Modem::validate_soft_restart_result(Ok("\r\n+CME ERROR: 3\r\n".to_string()));

        assert!(result.is_err());
    }
}

#[cfg(all(test, unix))]
#[path = "core/serial_tests.rs"]
mod serial_tests;
