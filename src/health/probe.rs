use std::{collections::BTreeSet, future::Future, pin::Pin};

use anyhow::Result;

use crate::{
    config::SmsStorage,
    health::state::FailureReason,
    modem::{SmsStorageStatus, SmsType},
    ModemManagerRef,
};

pub type HealthFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthCheckResult {
    failed_reasons: BTreeSet<FailureReason>,
}

impl HealthCheckResult {
    pub fn success() -> Self {
        Self {
            failed_reasons: BTreeSet::new(),
        }
    }

    pub fn failure<I>(reasons: I) -> Self
    where
        I: IntoIterator<Item = FailureReason>,
    {
        Self {
            failed_reasons: reasons.into_iter().collect(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.failed_reasons.is_empty()
    }

    pub fn failed_reasons(&self) -> &BTreeSet<FailureReason> {
        &self.failed_reasons
    }
}

pub trait HealthProbe: Send + Sync {
    fn sim_ids(&self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;
    fn run_checks<'a>(&'a self, sim_id: &'a str) -> HealthFuture<'a, Result<HealthCheckResult>>;
}

pub struct ModemHealthProbe {
    modem_manager: ModemManagerRef,
}

impl ModemHealthProbe {
    pub fn new(modem_manager: ModemManagerRef) -> Self {
        Self { modem_manager }
    }

    fn configured_storage_code(storage: SmsStorage) -> &'static str {
        match storage {
            SmsStorage::SIM => "SM",
            SmsStorage::ME => "ME",
            SmsStorage::MT => "MT",
        }
    }

    fn classify_storage(
        configured_storage: Option<SmsStorage>,
        storage: Option<&SmsStorageStatus>,
    ) -> Option<FailureReason> {
        let storage = match storage {
            Some(storage) => storage,
            None => return Some(FailureReason::SmsStorageUnavailable),
        };
        let expected_receive_storage = configured_storage
            .map(Self::configured_storage_code)
            .unwrap_or(storage.receive_storage.as_str());

        if storage.receive_storage != expected_receive_storage || storage.receive_total == 0 {
            return Some(FailureReason::SmsStorageUnavailable);
        }

        if storage.receive_used >= storage.receive_total {
            return Some(FailureReason::SmsStorageFull);
        }

        None
    }
}

impl HealthProbe for ModemHealthProbe {
    fn sim_ids(&self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move { self.modem_manager.get_sim_ids().await })
    }

    fn run_checks<'a>(&'a self, sim_id: &'a str) -> HealthFuture<'a, Result<HealthCheckResult>> {
        Box::pin(async move {
            let mut failed_reasons = BTreeSet::new();

            if self.modem_manager.probe_at(sim_id).await.is_err() {
                failed_reasons.insert(FailureReason::AtUnreachable);
                return Ok(HealthCheckResult::failure(failed_reasons));
            }

            let sim_ready = self
                .modem_manager
                .get_sim_status(sim_id)
                .await?
                .map(|status| status.trim().eq_ignore_ascii_case("READY"))
                .unwrap_or(false);
            if !sim_ready {
                failed_reasons.insert(FailureReason::SimNotReady);
            }

            let network_registered = self
                .modem_manager
                .check_network_registration(sim_id)
                .await?
                .map(|status| status.is_registered())
                .unwrap_or(false);
            if !network_registered {
                failed_reasons.insert(FailureReason::NetworkNotRegistered);
            }

            let configured_storage = self.modem_manager.get_configured_sms_storage(sim_id).await;
            let storage = match self.modem_manager.get_sms_storage_overview(sim_id).await {
                Ok(storage) => storage,
                Err(_) => {
                    failed_reasons.insert(FailureReason::SmsStorageUnavailable);
                    return Ok(HealthCheckResult::failure(failed_reasons));
                }
            };
            let Some(storage) = storage else {
                failed_reasons.insert(FailureReason::SmsStorageUnavailable);
                return Ok(HealthCheckResult::failure(failed_reasons));
            };

            if let Some(reason) = Self::classify_storage(configured_storage, Some(&storage)) {
                failed_reasons.insert(reason);
            }

            if self
                .modem_manager
                .read_sms(sim_id, SmsType::RecUnread)
                .await
                .is_err()
            {
                failed_reasons.insert(FailureReason::ReadSmsFailed);
            }

            Ok(if failed_reasons.is_empty() {
                HealthCheckResult::success()
            } else {
                HealthCheckResult::failure(failed_reasons)
            })
        })
    }
}

#[cfg(test)]
pub(crate) fn assert_d_probe_uses_configured_receive_storage_semantics() {
    let matching_full = SmsStorageStatus {
        read_storage: "ME".to_string(),
        read_used: 1,
        read_total: 10,
        write_storage: "ME".to_string(),
        write_used: 1,
        write_total: 10,
        receive_storage: "ME".to_string(),
        receive_used: 10,
        receive_total: 10,
    };
    let mismatched_receive = SmsStorageStatus {
        receive_storage: "SM".to_string(),
        ..matching_full.clone()
    };

    assert_eq!(
        ModemHealthProbe::classify_storage(Some(SmsStorage::ME), Some(&matching_full)),
        Some(FailureReason::SmsStorageFull)
    );
    assert_eq!(
        ModemHealthProbe::classify_storage(Some(SmsStorage::ME), Some(&mismatched_receive)),
        Some(FailureReason::SmsStorageUnavailable)
    );
    assert_eq!(
        ModemHealthProbe::classify_storage(None, Some(&matching_full)),
        Some(FailureReason::SmsStorageFull)
    );
    assert_eq!(
        ModemHealthProbe::classify_storage(Some(SmsStorage::ME), None),
        Some(FailureReason::SmsStorageUnavailable)
    );
}
