use std::{sync::Arc, time::Duration};

use log::{error, warn};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};

use crate::health::state::{HealthSnapshot, RecoveryAction};

use super::{
    probe::HealthProbe,
    recovery::{RecoveryExecutor, RecoveryPlan},
};

pub struct HealthSupervisor {
    probe: Arc<dyn HealthProbe>,
    recovery: Arc<dyn RecoveryExecutor>,
    failure_threshold: u64,
    recovery_plan: RecoveryPlan,
    snapshot: Arc<RwLock<HealthSnapshot>>,
}

impl HealthSupervisor {
    pub fn new(
        probe: Arc<dyn HealthProbe>,
        recovery: Arc<dyn RecoveryExecutor>,
        failure_threshold: u64,
        recovery_plan: RecoveryPlan,
    ) -> Self {
        Self {
            probe,
            recovery,
            failure_threshold,
            recovery_plan,
            snapshot: Arc::new(RwLock::new(HealthSnapshot::new())),
        }
    }

    pub async fn snapshot(&self) -> HealthSnapshot {
        self.snapshot.read().await.clone()
    }

    pub async fn replace_snapshot(&self, snapshot: HealthSnapshot) {
        *self.snapshot.write().await = snapshot;
    }

    pub async fn run_probe_cycle(&self) -> HealthSnapshot {
        let current = self.snapshot().await;

        let next = match self.probe.run_checks().await {
            Ok(result) if result.is_healthy() => current.record_success(),
            Ok(result) => {
                let failed = current.record_failure(
                    self.failure_threshold,
                    result.failed_reasons().iter().copied(),
                    Some(RecoveryAction::ReinitializeModem),
                );

                if matches!(
                    failed.current_status,
                    crate::health::state::HealthStatus::Recovering
                ) {
                    match self.recovery.execute(&self.recovery_plan).await {
                        Ok(()) => match self.probe.run_checks().await {
                            Ok(follow_up) if follow_up.is_healthy() => failed.record_success(),
                            Ok(_) => failed,
                            Err(err) => {
                                warn!("Health probe failed after recovery: {}", err);
                                failed
                            }
                        },
                        Err(err) => {
                            error!("Health recovery failed: {}", err);
                            failed
                        }
                    }
                } else {
                    failed
                }
            }
            Err(err) => {
                warn!("Health probe execution failed: {}", err);
                current.record_failure(
                    self.failure_threshold,
                    [crate::health::state::FailureReason::AtUnreachable],
                    Some(RecoveryAction::ReinitializeModem),
                )
            }
        };

        self.replace_snapshot(next.clone()).await;
        next
    }

    pub fn spawn_worker(self: Arc<Self>, interval: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                self.run_probe_cycle().await;
                sleep(interval).await;
            }
        })
    }
}
