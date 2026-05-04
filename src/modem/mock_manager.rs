use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::SseManager;
use crate::config::{AppConfig, SmsStorage};
use crate::db::{Contact, SimCard, Sms, SmsStatus};
use crate::webhook;

use super::types::{
    ModemInfo, NetworkRegistrationStatus, OperatorInfo, SignalQuality, SmsStorageStatus, SmsType,
};

pub struct MockModem {
    pub com_port: String,
    pub baud_rate: u32,
}

pub struct ModemManager {
    modems: Arc<RwLock<HashMap<String, Arc<MockModem>>>>,
    sim_cards_cache: Arc<RwLock<HashMap<String, SimCard>>>,
    configured_sms_storage: Arc<RwLock<HashMap<String, SmsStorage>>>,
}

impl ModemManager {
    fn sms_storage_code(storage: SmsStorage) -> &'static str {
        match storage {
            SmsStorage::SIM => "SM",
            SmsStorage::ME => "ME",
            SmsStorage::MT => "MT",
        }
    }

    fn configured_sms_storage_for(
        configured_sms_storage: &HashMap<String, SmsStorage>,
        sim_id: &str,
    ) -> SmsStorage {
        configured_sms_storage
            .get(sim_id)
            .copied()
            .unwrap_or(SmsStorage::SIM)
    }

    fn sms_storage_status_response(storage: SmsStorage) -> String {
        let storage = Self::sms_storage_code(storage);
        format!("+CPMS: \"{0}\",5,100,\"{0}\",5,100,\"{0}\",5,100", storage)
    }

    pub async fn initialize(config: &AppConfig) -> anyhow::Result<Self> {
        let mut modems = HashMap::new();
        let mut sim_ids = Vec::new();
        let mut configured_sms_storage = HashMap::new();

        if config.devices.is_empty() {
            for index in 0..2 {
                let sim_id = format!("mock_sim_{}", index + 1);
                sim_ids.push(sim_id.clone());
                if let Some(storage) = config.settings.sms_storage {
                    configured_sms_storage.insert(sim_id.clone(), storage);
                }
                modems.insert(
                    sim_id,
                    Arc::new(MockModem {
                        com_port: format!("mock://{}", index + 1),
                        baud_rate: 115200,
                    }),
                );
            }
        } else {
            for (index, device) in config.devices.iter().enumerate() {
                let sim_id = format!("mock_sim_{}", index + 1);
                sim_ids.push(sim_id.clone());
                if let Some(storage) = device.sms_storage.or(config.settings.sms_storage) {
                    configured_sms_storage.insert(sim_id.clone(), storage);
                }
                modems.insert(
                    sim_id,
                    Arc::new(MockModem {
                        com_port: if device.com_port.trim().is_empty() {
                            format!("mock://{}", index + 1)
                        } else {
                            device.com_port.clone()
                        },
                        baud_rate: if device.baud_rate == 0 {
                            115200
                        } else {
                            device.baud_rate
                        },
                    }),
                );
            }
        }

        Self::ensure_sim_cards(&sim_ids).await?;

        let manager = Self {
            modems: Arc::new(RwLock::new(modems)),
            sim_cards_cache: Arc::new(RwLock::new(HashMap::new())),
            configured_sms_storage: Arc::new(RwLock::new(configured_sms_storage)),
        };

        manager.init_sim_cache().await?;
        manager.seed_mock_data(&sim_ids).await?;

        Ok(manager)
    }

    async fn ensure_sim_cards(sim_ids: &[String]) -> anyhow::Result<()> {
        for (index, sim_id) in sim_ids.iter().enumerate() {
            let imsi = Some(format!("00101{:010}", index + 1));
            let phone_number = Some(format!("+1555000{:03}", index + 1));
            let _ = SimCard::find_or_create_with_phone(sim_id, imsi, phone_number).await?;
        }
        Ok(())
    }

    async fn init_sim_cache(&self) -> anyhow::Result<()> {
        let sim_ids: Vec<String> = self.modems.read().await.keys().cloned().collect();
        let sim_id_refs: Vec<&str> = sim_ids.iter().map(|id| id.as_str()).collect();
        let sim_cards = SimCard::get_by_ids(&sim_id_refs).await?;

        let mut cache = self.sim_cards_cache.write().await;
        *cache = sim_cards;
        Ok(())
    }

    async fn seed_mock_data(&self, sim_ids: &[String]) -> anyhow::Result<()> {
        if Sms::count().await? > 0 {
            return Ok(());
        }

        let mut contacts = Contact::query_all().await?;
        if contacts.is_empty() {
            contacts = vec![
                Contact {
                    id: Uuid::new_v4().to_string(),
                    name: "Alice".to_string(),
                },
                Contact {
                    id: Uuid::new_v4().to_string(),
                    name: "Bob".to_string(),
                },
                Contact {
                    id: Uuid::new_v4().to_string(),
                    name: "Support".to_string(),
                },
            ];

            for contact in &contacts {
                contact.insert().await?;
            }
        }

        if contacts.is_empty() {
            return Ok(());
        }

        let sim_primary = sim_ids
            .get(0)
            .cloned()
            .unwrap_or_else(|| "mock_sim_1".to_string());
        let sim_secondary = sim_ids
            .get(1)
            .cloned()
            .unwrap_or_else(|| sim_primary.clone());

        let now = Utc::now().naive_utc();
        let sample_messages = vec![
            (
                0,
                false,
                5,
                "Hey, can you send the status report?",
                sim_primary.clone(),
            ),
            (0, true, 4, "Sure, sending it now.", sim_primary.clone()),
            (1, false, 12, "Lunch at 12:30?", sim_secondary.clone()),
            (1, true, 10, "Sounds good.", sim_secondary.clone()),
            (
                2,
                false,
                30,
                "Welcome to SMS Gateway! Reply HELP for options.",
                sim_primary.clone(),
            ),
        ];

        for (contact_index, send, minutes_ago, message, sim_id) in sample_messages {
            if let Some(contact) = contacts.get(contact_index) {
                let sms = Sms {
                    id: 0,
                    contact_id: contact.id.clone(),
                    timestamp: now - Duration::minutes(minutes_ago),
                    message: message.to_string(),
                    sim_id,
                    send,
                    status: if send {
                        SmsStatus::Read
                    } else {
                        SmsStatus::Unread
                    },
                };
                let _ = sms.insert().await?;
            }
        }

        Ok(())
    }

    pub async fn get_sim_ids(&self) -> Vec<String> {
        self.modems.read().await.keys().cloned().collect()
    }

    pub async fn get_modem(&self, sim_id: &str) -> Option<Arc<MockModem>> {
        self.modems.read().await.get(sim_id).cloned()
    }

    pub async fn send_sms(
        &self,
        sim_id: &str,
        contact: &Contact,
        message: &str,
    ) -> anyhow::Result<(i64, String)> {
        let sms = Sms {
            id: 0,
            contact_id: contact.id.clone(),
            timestamp: Utc::now().naive_utc(),
            message: message.to_string(),
            sim_id: sim_id.to_string(),
            send: true,
            status: SmsStatus::Read,
        };

        let sms_id = sms.insert().await?;
        Ok((sms_id, contact.id.clone()))
    }

    pub async fn read_sms(
        &self,
        _sim_id: &str,
        _sms_type: SmsType,
    ) -> anyhow::Result<Vec<crate::db::ModemSMS>> {
        Ok(Vec::new())
    }

    pub async fn read_sms_async_insert(
        &self,
        _sim_id: &str,
        _sms_type: SmsType,
        _sse_manager: Arc<SseManager>,
        _webhook_manager: Option<webhook::WebhookManager>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn read_sms_sync_insert(
        &self,
        _sim_id: &str,
        _sms_type: SmsType,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn read_all_sms_async(
        &self,
        _sms_type: SmsType,
        _sse_manager: Arc<SseManager>,
        _webhook_manager: Option<webhook::WebhookManager>,
    ) {
    }

    pub async fn get_signal_quality(&self, _sim_id: &str) -> anyhow::Result<Option<SignalQuality>> {
        Ok(SignalQuality::from_response("+CSQ: 20,0"))
    }

    pub async fn check_network_registration(
        &self,
        _sim_id: &str,
    ) -> anyhow::Result<Option<NetworkRegistrationStatus>> {
        Ok(NetworkRegistrationStatus::from_response(
            "+CREG: 0,1,\"1A2B\",\"1A2B\"",
        ))
    }

    pub async fn check_operator(&self, _sim_id: &str) -> anyhow::Result<Option<OperatorInfo>> {
        Ok(OperatorInfo::from_response(
            "+COPS: 0,0,\"MockTel\",\"00101\"",
        ))
    }

    pub async fn get_modem_model(&self, _sim_id: &str) -> anyhow::Result<Option<ModemInfo>> {
        Ok(ModemInfo::from_response("Mock Modem 1.0"))
    }

    pub async fn get_sms_center(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("+15551230000".to_string()))
    }

    pub async fn get_network_info(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("Mock LTE".to_string()))
    }

    pub async fn get_sim_status(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("READY".to_string()))
    }

    pub async fn get_memory_status(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some(
            "+CPMS: \"SM\",5,100,\"SM\",5,100,\"SM\",5,100".to_string(),
        ))
    }

    pub async fn get_temperature_info(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("28C".to_string()))
    }

    pub async fn set_sms_storage(
        &self,
        sim_id: &str,
        sms_storage: SmsStorage,
    ) -> anyhow::Result<()> {
        self.configured_sms_storage
            .write()
            .await
            .insert(sim_id.to_string(), sms_storage);
        Ok(())
    }

    pub async fn reapply_configured_sms_storage(&self, sim_id: &str) -> anyhow::Result<()> {
        if self
            .configured_sms_storage
            .read()
            .await
            .contains_key(sim_id)
        {
            return Ok(());
        }

        Ok(())
    }

    pub async fn get_configured_sms_storage(&self, sim_id: &str) -> Option<SmsStorage> {
        self.configured_sms_storage
            .read()
            .await
            .get(sim_id)
            .copied()
    }

    pub async fn get_sms_storage_status(&self, sim_id: &str) -> anyhow::Result<Option<String>> {
        let configured_sms_storage = self.configured_sms_storage.read().await;
        let storage = Self::configured_sms_storage_for(&configured_sms_storage, sim_id);
        Ok(Some(Self::sms_storage_status_response(storage)))
    }

    pub async fn probe_at(&self, _sim_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn reinitialize_runtime(&self, _sim_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn soft_restart(&self, _sim_id: &str) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn get_sms_storage_overview(
        &self,
        sim_id: &str,
    ) -> anyhow::Result<Option<SmsStorageStatus>> {
        let configured_sms_storage = self.configured_sms_storage.read().await;
        let storage = Self::configured_sms_storage_for(&configured_sms_storage, sim_id);
        Ok(SmsStorageStatus::from_response(
            &Self::sms_storage_status_response(storage),
        ))
    }

    pub async fn get_sim_card_cached(&self, sim_id: &str) -> Option<SimCard> {
        self.sim_cards_cache.read().await.get(sim_id).cloned()
    }

    pub async fn update_sim_cache(&self, sim_card: SimCard) {
        let mut cache = self.sim_cards_cache.write().await;
        cache.insert(sim_card.id.clone(), sim_card);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soft_restart_status_response_uses_configured_storage() {
        let mut configured_sms_storage = HashMap::new();
        configured_sms_storage.insert("sim-1".to_string(), SmsStorage::ME);

        let storage = ModemManager::configured_sms_storage_for(&configured_sms_storage, "sim-1");
        let response = ModemManager::sms_storage_status_response(storage);

        assert_eq!(response, "+CPMS: \"ME\",5,100,\"ME\",5,100,\"ME\",5,100");
    }

    #[test]
    fn soft_restart_status_response_defaults_to_sim_storage() {
        let configured_sms_storage = HashMap::new();

        let storage =
            ModemManager::configured_sms_storage_for(&configured_sms_storage, "missing-sim");
        let response = ModemManager::sms_storage_status_response(storage);
        let overview = SmsStorageStatus::from_response(&response).expect("expected mock overview");

        assert_eq!(response, "+CPMS: \"SM\",5,100,\"SM\",5,100,\"SM\",5,100");
        assert_eq!(overview.read_storage, "SM");
        assert_eq!(overview.write_storage, "SM");
        assert_eq!(overview.receive_storage, "SM");
    }

    #[tokio::test]
    async fn reapply_configured_sms_storage_uses_per_sim_configuration() {
        let manager = ModemManager {
            modems: Arc::new(RwLock::new(HashMap::new())),
            sim_cards_cache: Arc::new(RwLock::new(HashMap::new())),
            configured_sms_storage: Arc::new(RwLock::new(HashMap::from([
                ("sim-1".to_string(), SmsStorage::ME),
                ("sim-2".to_string(), SmsStorage::MT),
            ]))),
        };

        manager
            .reapply_configured_sms_storage("sim-1")
            .await
            .expect("expected storage reapply to succeed");
        manager
            .reapply_configured_sms_storage("sim-2")
            .await
            .expect("expected storage reapply to succeed");

        let sim1 = manager
            .get_sms_storage_overview("sim-1")
            .await
            .expect("expected sim-1 storage overview")
            .expect("expected sim-1 storage state");
        let sim2 = manager
            .get_sms_storage_overview("sim-2")
            .await
            .expect("expected sim-2 storage overview")
            .expect("expected sim-2 storage state");

        assert_eq!(sim1.read_storage, "ME");
        assert_eq!(sim2.read_storage, "MT");
    }
}
