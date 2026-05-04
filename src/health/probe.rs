use std::{collections::BTreeSet, future::Future, pin::Pin};

use anyhow::Result;

use crate::{health::state::FailureReason, modem::SmsStorageStatus, ModemManagerRef};

pub type HealthFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeCheck {
    AtReachable,
    SimReady,
    NetworkRegistered,
    SmsStorageAvailable,
    SmsStorageNotFull,
}

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
    fn run_checks(&self) -> HealthFuture<'_, Result<HealthCheckResult>>;
}

pub struct ModemHealthProbe {
    modem_manager: ModemManagerRef,
}

impl ModemHealthProbe {
    pub fn new(modem_manager: ModemManagerRef) -> Self {
        Self { modem_manager }
    }

    fn storage_is_full(storage: &SmsStorageStatus) -> bool {
        [
            (storage.read_used, storage.read_total),
            (storage.write_used, storage.write_total),
            (storage.receive_used, storage.receive_total),
        ]
        .into_iter()
        .any(|(used, total)| total > 0 && used >= total)
    }
}

impl HealthProbe for ModemHealthProbe {
    fn run_checks(&self) -> HealthFuture<'_, Result<HealthCheckResult>> {
        Box::pin(async move {
            let mut failed_reasons = BTreeSet::new();

            for sim_id in self.modem_manager.get_sim_ids().await {
                if self.modem_manager.probe_at(&sim_id).await.is_err() {
                    failed_reasons.insert(FailureReason::AtUnreachable);
                    continue;
                }

                let sim_ready = self
                    .modem_manager
                    .get_sim_status(&sim_id)
                    .await?
                    .map(|status| status.trim().eq_ignore_ascii_case("READY"))
                    .unwrap_or(false);
                if !sim_ready {
                    failed_reasons.insert(FailureReason::SimNotReady);
                }

                let network_registered = self
                    .modem_manager
                    .check_network_registration(&sim_id)
                    .await?
                    .map(|status| status.is_registered())
                    .unwrap_or(false);
                if !network_registered {
                    failed_reasons.insert(FailureReason::NetworkNotRegistered);
                }

                let storage = self.modem_manager.get_sms_storage_overview(&sim_id).await?;
                let Some(storage) = storage else {
                    failed_reasons.insert(FailureReason::SmsStorageUnavailable);
                    continue;
                };

                if Self::storage_is_full(&storage) {
                    failed_reasons.insert(FailureReason::SmsStorageFull);
                }
            }

            Ok(if failed_reasons.is_empty() {
                HealthCheckResult::success()
            } else {
                HealthCheckResult::failure(failed_reasons)
            })
        })
    }
}
