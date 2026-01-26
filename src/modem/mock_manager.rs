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
    ModemInfo, NetworkRegistrationStatus, OperatorInfo, SignalQuality, SmsType,
};

pub struct MockModem {
    pub com_port: String,
    pub baud_rate: u32,
}

pub struct ModemManager {
    modems: Arc<RwLock<HashMap<String, Arc<MockModem>>>>,
    sim_cards_cache: Arc<RwLock<HashMap<String, SimCard>>>,
}

impl ModemManager {
    pub async fn initialize(config: &AppConfig) -> anyhow::Result<Self> {
        let mut modems = HashMap::new();
        let mut sim_ids = Vec::new();

        if config.devices.is_empty() {
            for index in 0..2 {
                let sim_id = format!("mock_sim_{}", index + 1);
                sim_ids.push(sim_id.clone());
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
            (
                0,
                true,
                4,
                "Sure, sending it now.",
                sim_primary.clone(),
            ),
            (
                1,
                false,
                12,
                "Lunch at 12:30?",
                sim_secondary.clone(),
            ),
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

    pub async fn read_sms(&self, _sim_id: &str, _sms_type: SmsType) -> anyhow::Result<Vec<crate::db::ModemSMS>> {
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
        Ok(NetworkRegistrationStatus::from_response("+CREG: 0,1,\"1A2B\",\"1A2B\""))
    }

    pub async fn check_operator(&self, _sim_id: &str) -> anyhow::Result<Option<OperatorInfo>> {
        Ok(OperatorInfo::from_response("+COPS: 0,0,\"MockTel\",\"00101\""))
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
        Ok(Some("+CPMS: \"SM\",5,100,\"SM\",5,100,\"SM\",5,100".to_string()))
    }

    pub async fn get_temperature_info(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("28C".to_string()))
    }

    pub async fn set_sms_storage(
        &self,
        _sim_id: &str,
        _sms_storage: SmsStorage,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn get_sms_storage_status(&self, _sim_id: &str) -> anyhow::Result<Option<String>> {
        Ok(Some("+CPMS: \"SM\",5,100,\"SM\",5,100,\"SM\",5,100".to_string()))
    }

    pub async fn get_sim_card_cached(&self, sim_id: &str) -> Option<SimCard> {
        self.sim_cards_cache.read().await.get(sim_id).cloned()
    }

    pub async fn update_sim_cache(&self, sim_card: SimCard) {
        let mut cache = self.sim_cards_cache.write().await;
        cache.insert(sim_card.id.clone(), sim_card);
    }
}
