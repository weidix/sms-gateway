use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, warn};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};

use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus};

use super::{
    alert::{AlertGate, HealthAlertSink},
    probe::HealthProbe,
    recovery::{RecoveryExecutor, RecoveryPlan, RecoveryStep},
};

pub struct HealthSupervisor {
    probe: Arc<dyn HealthProbe>,
    recovery: Arc<dyn RecoveryExecutor>,
    alert_sink: Option<Arc<dyn HealthAlertSink>>,
    failure_threshold: u64,
    recovery_plan: RecoveryPlan,
    alert_gates: Arc<RwLock<BTreeMap<String, AlertGate>>>,
    snapshots: Arc<RwLock<BTreeMap<String, HealthSnapshot>>>,
}

impl HealthSupervisor {
    pub fn new(
        probe: Arc<dyn HealthProbe>,
        recovery: Arc<dyn RecoveryExecutor>,
        failure_threshold: u64,
        recovery_plan: RecoveryPlan,
    ) -> Self {
        Self::with_optional_alert_sink(probe, recovery, failure_threshold, recovery_plan, None)
    }

    pub fn with_alert_sink(
        probe: Arc<dyn HealthProbe>,
        recovery: Arc<dyn RecoveryExecutor>,
        failure_threshold: u64,
        recovery_plan: RecoveryPlan,
        alert_sink: Arc<dyn HealthAlertSink>,
    ) -> Self {
        Self::with_optional_alert_sink(
            probe,
            recovery,
            failure_threshold,
            recovery_plan,
            Some(alert_sink),
        )
    }

    fn with_optional_alert_sink(
        probe: Arc<dyn HealthProbe>,
        recovery: Arc<dyn RecoveryExecutor>,
        failure_threshold: u64,
        recovery_plan: RecoveryPlan,
        alert_sink: Option<Arc<dyn HealthAlertSink>>,
    ) -> Self {
        Self {
            probe,
            recovery,
            alert_sink,
            failure_threshold,
            recovery_plan,
            alert_gates: Arc::new(RwLock::new(BTreeMap::new())),
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
            self.replace_snapshot_and_emit(&sim_id, next).await;
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
                self.process_failed_snapshot(sim_id, current, result.failed_reasons().clone())
                    .await
            }
            Err(err) => {
                warn!("Health probe execution failed for {}: {}", sim_id, err);
                self.process_failed_snapshot(
                    sim_id,
                    current,
                    BTreeSet::from([FailureReason::AtUnreachable]),
                )
                .await
            }
        }
    }

    async fn recover_sim(&self, sim_id: &str, snapshot: HealthSnapshot) -> HealthSnapshot {
        let mut recovering = snapshot;

        for step in self.recovery_plan.steps() {
            if let Some(action) = step.recovery_action() {
                recovering = recovering.record_recovery_action(action);
                self.replace_snapshot_and_emit(sim_id, recovering.clone())
                    .await;
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

    async fn process_failed_snapshot(
        &self,
        sim_id: &str,
        current: HealthSnapshot,
        failure_reasons: BTreeSet<FailureReason>,
    ) -> HealthSnapshot {
        let failed = current.record_failure(
            self.failure_threshold,
            failure_reasons.iter().copied(),
            None,
        );

        if matches!(failed.current_status, HealthStatus::Recovering) {
            self.recover_sim(sim_id, failed).await
        } else {
            failed
        }
    }

    async fn replace_snapshot_and_emit(&self, sim_id: &str, snapshot: HealthSnapshot) {
        self.replace_snapshot(sim_id, snapshot.clone()).await;

        if let Some(alert_sink) = &self.alert_sink {
            if self.should_emit_alert(sim_id, &snapshot).await {
                alert_sink.emit(sim_id, &snapshot).await;
            }
        }
    }

    async fn should_emit_alert(&self, sim_id: &str, snapshot: &HealthSnapshot) -> bool {
        let mut gates = self.alert_gates.write().await;
        let gate = gates.entry(sim_id.to_string()).or_default();
        gate.should_emit(snapshot)
    }
}

impl HealthSupervisor {
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
