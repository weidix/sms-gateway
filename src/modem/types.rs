use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Copy)]
pub enum SmsType {
    RecUnread,
    RecRead,
    StoUnsent,
    StoSent,
    All,
}

impl SmsType {
    pub fn to_at_command_pdu(&self) -> u8 {
        match self {
            SmsType::RecUnread => 0,
            SmsType::RecRead => 1,
            SmsType::StoUnsent => 2,
            SmsType::StoSent => 3,
            SmsType::All => 4,
        }
    }
}

#[derive(Debug)]
pub struct ATCommand {
    pub command: String,
    pub response_tx: tokio::sync::oneshot::Sender<Result<String, io::Error>>,
    pub _priority: u8,
    pub retries: u32,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalQuality {
    rssi: i32,
    ber: i32,
}

impl SignalQuality {
    pub fn from_response(response: &str) -> Option<Self> {
        response
            .lines()
            .find(|line| line.trim().starts_with("+CSQ:"))
            .and_then(|line| {
                let data = line.split(':').nth(1)?;
                let parts: Vec<&str> = data.split(',').collect();

                if parts.len() >= 2 {
                    Some(SignalQuality {
                        rssi: parts[0].trim().parse().ok()?,
                        ber: parts[1].trim().parse().ok()?,
                    })
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRegistrationStatus {
    status: String,
    location_area_code: Option<String>,
    cell_id: Option<String>,
}

impl NetworkRegistrationStatus {
    pub fn from_response(response: &str) -> Option<Self> {
        response
            .lines()
            .find(|line| line.trim().starts_with("+CREG:"))
            .and_then(|line| {
                let data = line.split(':').nth(1)?;
                let parts: Vec<&str> = data.split(',').collect();

                if parts.len() >= 2 {
                    Some(NetworkRegistrationStatus {
                        status: parts[1].trim().trim_matches('"').to_string(),
                        location_area_code: parts
                            .get(2)
                            .map(|s| s.trim().trim_matches('"').to_string()),
                        cell_id: parts.get(3).map(|s| s.trim().trim_matches('"').to_string()),
                    })
                } else {
                    None
                }
            })
    }

    pub fn is_registered(&self) -> bool {
        matches!(self.status.as_str(), "1" | "5")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsStorageStatus {
    pub read_storage: String,
    pub read_used: u32,
    pub read_total: u32,
    pub write_storage: String,
    pub write_used: u32,
    pub write_total: u32,
    pub receive_storage: String,
    pub receive_used: u32,
    pub receive_total: u32,
}

impl SmsStorageStatus {
    pub fn from_response(response: &str) -> Option<Self> {
        response
            .lines()
            .find(|line| line.trim().starts_with("+CPMS:"))
            .and_then(|line| {
                let data = line.split(':').nth(1)?;
                let parts: Vec<&str> = data.split(',').collect();

                if parts.len() < 9 {
                    return None;
                }

                Some(SmsStorageStatus {
                    read_storage: parts[0].trim().trim_matches('"').to_string(),
                    read_used: parts[1].trim().parse().ok()?,
                    read_total: parts[2].trim().parse().ok()?,
                    write_storage: parts[3].trim().trim_matches('"').to_string(),
                    write_used: parts[4].trim().parse().ok()?,
                    write_total: parts[5].trim().parse().ok()?,
                    receive_storage: parts[6].trim().trim_matches('"').to_string(),
                    receive_used: parts[7].trim().parse().ok()?,
                    receive_total: parts[8].trim().parse().ok()?,
                })
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorInfo {
    operator_name: String,
    operator_id: String,
    registration_status: String,
}

impl OperatorInfo {
    pub fn from_response(response: &str) -> Option<Self> {
        response
            .lines()
            .find(|line| line.trim().starts_with("+COPS:"))
            .and_then(|line| {
                let data = line.split(':').nth(1)?;
                let parts: Vec<&str> = data.split(',').collect();

                if parts.len() >= 3 {
                    Some(OperatorInfo {
                        registration_status: parts[0].trim().to_string(),
                        operator_name: parts[2].trim_matches('"').to_string(),
                        operator_id: parts
                            .get(3)
                            .map(|s| s.trim_matches('"').to_string())
                            .unwrap_or_else(|| parts[2].trim_matches('"').to_string()),
                    })
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModemInfo {
    model: String,
}

impl ModemInfo {
    pub fn from_response(response: &str) -> Option<Self> {
        let model = response.trim().to_string();
        if !model.is_empty() && !model.contains("ERROR") {
            Some(ModemInfo { model })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NetworkRegistrationStatus, SmsStorageStatus};

    #[test]
    fn parses_sms_storage_status_from_cpms() {
        let status = SmsStorageStatus::from_response(
            "\r\n+CPMS: \"SM\",5,100,\"ME\",2,50,\"MT\",7,150\r\n\r\nOK\r\n",
        )
        .expect("expected CPMS status to parse");

        assert_eq!(status.read_storage, "SM");
        assert_eq!(status.read_used, 5);
        assert_eq!(status.read_total, 100);
        assert_eq!(status.write_storage, "ME");
        assert_eq!(status.write_used, 2);
        assert_eq!(status.write_total, 50);
        assert_eq!(status.receive_storage, "MT");
        assert_eq!(status.receive_used, 7);
        assert_eq!(status.receive_total, 150);
    }

    #[test]
    fn network_registration_treats_home_and_roaming_as_registered() {
        let home = NetworkRegistrationStatus::from_response("+CREG: 0,1,\"1A2B\",\"1A2B\"")
            .expect("expected home registration to parse");
        let roaming = NetworkRegistrationStatus::from_response("+CREG: 0,5,\"1A2B\",\"1A2B\"")
            .expect("expected roaming registration to parse");
        let searching = NetworkRegistrationStatus::from_response("+CREG: 0,2")
            .expect("expected searching registration to parse");

        assert!(home.is_registered());
        assert!(roaming.is_registered());
        assert!(!searching.is_registered());
    }
}
