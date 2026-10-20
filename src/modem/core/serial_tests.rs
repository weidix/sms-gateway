use super::{ConnectionState, Modem, SmsStorage};
use std::{io, path::PathBuf, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::JoinHandle,
    time::timeout,
};
use tokio_serial::{SerialPort, SerialStream};

struct TestDevice {
    directory: PathBuf,
    path: PathBuf,
    slave: std::sync::Mutex<Option<SerialStream>>,
}

impl TestDevice {
    fn new() -> io::Result<(Self, SerialStream)> {
        let directory = std::env::temp_dir().join(format!("sms-serial-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory)?;
        let device = Self {
            path: directory.join("if02"),
            directory,
            slave: std::sync::Mutex::new(None),
        };
        let master = device.attach()?;
        Ok((device, master))
    }

    fn attach(&self) -> io::Result<SerialStream> {
        let (master, slave) = SerialStream::pair()?;
        let slave_path = slave
            .name()
            .ok_or_else(|| io::Error::other("PTY slave has no device path"))?;
        // A private symlink lets the test replace the device behind a fixed path
        // without changing host device nodes or requiring elevated privileges.
        if self.path.exists() || self.path.is_symlink() {
            std::fs::remove_file(&self.path)?;
        }
        std::os::unix::fs::symlink(slave_path, &self.path)?;
        *self.slave.lock().unwrap() = Some(slave);
        Ok(master)
    }

    async fn modem(&self) -> io::Result<Modem> {
        // serialport reserves baud rate 0 for PTYs (no modem-control ioctls).
        let modem = Modem::new(self.path.to_str().unwrap(), 0, "hotplug-test").await?;
        self.slave.lock().unwrap().take();
        Ok(modem)
    }
}

impl Drop for TestDevice {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_dir(&self.directory);
    }
}

async fn read_command(master: &mut SerialStream) -> String {
    timeout(Duration::from_secs(5), async {
        let mut command = Vec::new();
        loop {
            let byte = master.read_u8().await.unwrap();
            command.push(byte);
            if byte == b'\n' {
                return String::from_utf8(command).unwrap();
            }
        }
    })
    .await
    .expect("modem did not send expected command")
}

fn respond(master: SerialStream, commands: Vec<String>) -> JoinHandle<SerialStream> {
    tokio::spawn(async move {
        let mut master = master;
        for expected in commands {
            assert_eq!(read_command(&mut master).await, expected);
            master.write_all(b"\r\nOK\r\n").await.unwrap();
        }
        // Keep the peer open until the caller has consumed the final response.
        master
    })
}

fn initialization(storage: Option<SmsStorage>) -> Vec<String> {
    let mut commands: Vec<_> = Modem::init_commands()
        .into_iter()
        .map(|(command, _)| command.to_string())
        .collect();
    if let Some(storage) = storage {
        commands.push(Modem::sms_storage_command(storage));
    }
    commands
}

#[tokio::test]
async fn transport_error_closes_old_handle_and_next_commands_reopen_replacement() {
    let (device, old_master) = TestDevice::new().unwrap();
    let modem = device.modem().await.unwrap();
    let configure = respond(old_master, vec![Modem::sms_storage_command(SmsStorage::ME)]);
    modem.set_sms_storage(SmsStorage::ME).await.unwrap();
    let old_master = configure.await.unwrap();
    drop(old_master);

    assert!(timeout(Duration::from_secs(5), modem.probe_at())
        .await
        .unwrap()
        .is_err());
    {
        let connection = modem.serial_connection.lock().await;
        assert!(connection.stream.is_none());
        assert!(matches!(connection.state, ConnectionState::Disconnected));
    }

    let replacement = device.attach().unwrap();
    let mut commands = initialization(Some(SmsStorage::ME));
    commands.extend(["AT\r\n".to_string(), "AT\r\n".to_string()]);
    let responder = respond(replacement, commands);
    // Both queued callers must wait for one reopening and full initialization.
    let (first, second) = tokio::join!(modem.probe_at(), modem.probe_at());
    first.unwrap();
    second.unwrap();
    let _replacement = responder.await.unwrap();
    let connection = modem.serial_connection.lock().await;
    assert!(connection.stream.is_some());
    assert!(matches!(connection.state, ConnectionState::Connected));
}

#[tokio::test]
async fn runtime_reinitialization_reopens_even_when_old_handle_is_healthy() {
    let (device, mut old_master) = TestDevice::new().unwrap();
    let modem = device.modem().await.unwrap();
    let replacement = device.attach().unwrap();
    let mut commands = initialization(Some(SmsStorage::SIM));
    // SIM discovery has concurrent commands; empty OK replies avoid database use.
    let responder = tokio::spawn(async move {
        let mut replacement = replacement;
        for expected in commands.drain(..) {
            assert_eq!(read_command(&mut replacement).await, expected);
            replacement.write_all(b"\r\nOK\r\n").await.unwrap();
        }
        let mut sim_commands = Vec::new();
        for _ in 0..3 {
            sim_commands.push(read_command(&mut replacement).await);
            replacement.write_all(b"\r\nOK\r\n").await.unwrap();
        }
        sim_commands.sort();
        assert_eq!(sim_commands, ["AT+CCID\r\n", "AT+CIMI\r\n", "AT+CNUM\r\n"]);
        replacement
    });

    timeout(
        Duration::from_secs(10),
        modem.reinitialize_runtime(Some(SmsStorage::SIM)),
    )
    .await
    .unwrap()
    .unwrap();
    let _replacement = responder.await.unwrap();
    // Closing the slave can yield either EOF or a platform-specific PTY error.
    let mut buffer = [0u8; 1];
    let old_result = timeout(Duration::from_secs(1), old_master.read(&mut buffer))
        .await
        .expect("old serial handle was not closed");
    assert!(!matches!(old_result, Ok(bytes) if bytes > 0));
}

#[tokio::test]
async fn failed_initialization_keeps_connection_disconnected_and_allows_later_recovery() {
    let (device, old_master) = TestDevice::new().unwrap();
    let modem = device.modem().await.unwrap();
    modem.serial_connection.lock().await.disconnect();
    drop(old_master);
    let replacement = device.attach().unwrap();
    let rejecting = tokio::spawn(async move {
        let mut replacement = replacement;
        assert_eq!(read_command(&mut replacement).await, "ATE0\r\n");
        replacement.write_all(b"\r\nERROR\r\n").await.unwrap();
        replacement
    });
    {
        let mut connection = modem.serial_connection.lock().await;
        assert!(Modem::attempt_reconnection(
            &mut connection,
            &modem.com_port,
            modem.baud_rate,
            &modem.name,
        )
        .await
        .is_err());
        assert!(connection.stream.is_none());
        assert!(matches!(connection.state, ConnectionState::Disconnected));
    }
    let _rejected_peer = rejecting.await.unwrap();
    let replacement = device.attach().unwrap();
    let mut commands = initialization(None);
    commands.push("AT\r\n".to_string());
    let responder = respond(replacement, commands);
    modem.probe_at().await.unwrap();
    let _replacement = responder.await.unwrap();
}

#[tokio::test]
async fn missing_device_can_reappear_during_reconnection_retries() {
    let (device, old_master) = TestDevice::new().unwrap();
    let modem = device.modem().await.unwrap();
    modem.serial_connection.lock().await.disconnect();
    drop(old_master);
    std::fs::remove_file(&device.path).unwrap();

    let recovering = modem.probe_at();
    let reattach = async {
        // The first attempt fails on the absent path; a later attempt opens it.
        tokio::time::sleep(Duration::from_millis(50)).await;
        let replacement = device.attach().unwrap();
        let mut commands = initialization(None);
        commands.push("AT\r\n".to_string());
        respond(replacement, commands).await.unwrap()
    };
    let (result, _replacement) = tokio::join!(recovering, reattach);
    result.unwrap();
}
