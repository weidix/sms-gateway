use std::{collections::BTreeMap, sync::Arc, time::Duration};

use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, warn};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};

use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus};

use super::{
    probe::HealthProbe,
    recovery::{RecoveryExecutor, RecoveryPlan, RecoveryStep},
};

pub struct HealthSupervisor {
    probe: Arc<dyn HealthProbe>,
    recovery: Arc<dyn RecoveryExecutor>,
    failure_threshold: u64,
    recovery_plan: RecoveryPlan,
    snapshots: Arc<RwLock<BTreeMap<String, HealthSnapshot>>>,
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
            snapshots: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub async fn snapshot_for(&self, sim_id: &str) -> Option<HealthSnapshot> {
        self.snapshots.read().await.get(sim_id).cloned()
    }

    pub async fn replace_snapshot(&self, sim_id: &str, snapshot: HealthSnapshot) {
        self.snapshots
            .write()
            .await
            .insert(sim_id.to_string(), snapshot);
    }

    pub async fn run_probe_cycle(&self) -> BTreeMap<String, HealthSnapshot> {
        let mut tasks = FuturesUnordered::new();

        for sim_id in self.probe.sim_ids().await {
            let current = self
                .snapshot_for(&sim_id)
                .await
                .unwrap_or_else(HealthSnapshot::new);

            tasks.push(async move {
                (
                    sim_id.clone(),
                    self.run_probe_cycle_for_sim(&sim_id, current).await,
                )
            });
        }

        while let Some((sim_id, next)) = tasks.next().await {
            self.replace_snapshot(&sim_id, next).await;
        }

        self.snapshots.read().await.clone()
    }

    async fn run_probe_cycle_for_sim(
        &self,
        sim_id: &str,
        current: HealthSnapshot,
    ) -> HealthSnapshot {
        match self.probe.run_checks(sim_id).await {
            Ok(result) if result.is_healthy() => current.record_success(),
            Ok(result) => {
                let failed = current.record_failure(
                    self.failure_threshold,
                    result.failed_reasons().iter().copied(),
                    None,
                );

                if matches!(failed.current_status, HealthStatus::Recovering) {
                    self.recover_sim(sim_id, failed).await
                } else {
                    failed
                }
            }
            Err(err) => {
                warn!("Health probe execution failed for {}: {}", sim_id, err);
                current.record_failure(self.failure_threshold, [FailureReason::AtUnreachable], None)
            }
        }
    }

    async fn recover_sim(&self, sim_id: &str, snapshot: HealthSnapshot) -> HealthSnapshot {
        let mut recovering = snapshot;

        for step in self.recovery_plan.steps() {
            if let Some(action) = step.recovery_action() {
                recovering = recovering.record_recovery_action(action);
                self.replace_snapshot(sim_id, recovering.clone()).await;
            }

            if let Err(err) = self.recovery.execute_step(sim_id, *step).await {
                error!(
                    "Health recovery failed for {} at {:?}: {}",
                    sim_id, step, err
                );
                return recovering;
            }
        }

        match self.probe.run_checks(sim_id).await {
            Ok(follow_up) if follow_up.is_healthy() => recovering.record_success(),
            Ok(follow_up) => recovering.record_critical_failure(
                follow_up.failed_reasons().iter().copied(),
                step_last_action(self.recovery_plan.steps()),
            ),
            Err(err) => {
                warn!("Health probe failed after recovery for {}: {}", sim_id, err);
                recovering.record_critical_failure(
                    [FailureReason::AtUnreachable],
                    step_last_action(self.recovery_plan.steps()),
                )
            }
        }
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

fn step_last_action(steps: &[RecoveryStep]) -> Option<crate::health::state::RecoveryAction> {
    steps.iter().rev().find_map(|step| step.recovery_action())
}
