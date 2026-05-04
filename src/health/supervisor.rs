use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use futures::stream::{FuturesUnordered, StreamExt};
use log::{error, info, warn};
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

            info!(
                "Running health recovery for {}: step={:?}, failure_reasons={:?}, consecutive_failures={}",
                sim_id,
                step,
                recovering.failure_reasons,
                recovering.consecutive_failures
            );

            if let Err(err) = self.recovery.execute_step(sim_id, *step).await {
                error!(
                    "Health recovery failed for {} at {:?}: {}; failure_reasons={:?}; consecutive_failures={}",
                    sim_id,
                    step,
                    err,
                    recovering.failure_reasons,
                    recovering.consecutive_failures
                );
                return recovering;
            }
        }

        info!(
            "Health recovery steps finished for {}. Re-running probe with failure_reasons={:?}, consecutive_failures={}",
            sim_id,
            recovering.failure_reasons,
            recovering.consecutive_failures
        );

        match self.probe.run_checks(sim_id).await {
            Ok(follow_up) if follow_up.is_healthy() => {
                info!("Health recovery succeeded for {}", sim_id);
                recovering.record_success()
            }
            Ok(follow_up) => {
                warn!(
                    "Health recovery follow-up probe still failing for {}: failure_reasons={:?}",
                    sim_id,
                    follow_up.failed_reasons()
                );
                recovering.record_critical_failure(
                    follow_up.failed_reasons().iter().copied(),
                    step_last_action(self.recovery_plan.steps()),
                )
            }
            Err(err) => {
                warn!(
                    "Health probe failed after recovery for {}: {}; marking modem critical",
                    sim_id, err
                );
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
        let auto_recovery_allowed = allows_automatic_recovery(&failure_reasons);
        let failed = if auto_recovery_allowed {
            current.record_failure(
                self.failure_threshold,
                failure_reasons.iter().copied(),
                None,
            )
        } else {
            current.record_non_recoverable_failure(
                self.failure_threshold,
                failure_reasons.iter().copied(),
            )
        };

        warn!(
            "Health probe failed for {}: failure_reasons={:?}, consecutive_failures={}, threshold={}, status={:?}, auto_recovery={}",
            sim_id,
            failed.failure_reasons,
            failed.consecutive_failures,
            self.failure_threshold,
            failed.current_status,
            auto_recovery_allowed
        );

        if matches!(failed.current_status, HealthStatus::Recovering) {
            warn!(
                "Starting automatic health recovery for {} with plan {:?}",
                sim_id,
                self.recovery_plan.steps()
            );
            self.recover_sim(sim_id, failed).await
        } else {
            if !auto_recovery_allowed && failed.consecutive_failures >= self.failure_threshold {
                warn!(
                    "Automatic health recovery skipped for {} because failure_reasons={:?} are treated as non-recoverable",
                    sim_id,
                    failed.failure_reasons
                );
            }
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

fn allows_automatic_recovery(failure_reasons: &BTreeSet<FailureReason>) -> bool {
    !matches!(
        failure_reasons.iter().next(),
        Some(FailureReason::SmsStorageFull)
    ) || failure_reasons.len() != 1
}
