use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::sync::{Mutex, Notify};
use tokio::time::{timeout, Duration};

use crate::health::alert::HealthAlertSink;
use crate::health::probe::{HealthCheckResult, HealthProbe};
use crate::health::recovery::{RecoveryExecutor, RecoveryPlan, RecoveryStep};
use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus, RecoveryAction};
use crate::health::supervisor::HealthSupervisor;

pub(crate) fn assert_default_recovery_plan_is_stable() {
    assert_eq!(
        RecoveryPlan::default().steps(),
        &[
            RecoveryStep::ReinitializeRuntime,
            RecoveryStep::ReapplyStorageIfConfigured,
            RecoveryStep::SoftRestart,
            RecoveryStep::WaitWindow,
            RecoveryStep::ReinitializeRuntime,
        ]
    );
}

pub(crate) async fn assert_unhealthy_snapshot_returns_to_healthy_after_probe_success() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [("sim-1", vec![HealthCheckResult::success()])],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let supervisor = HealthSupervisor::new(probe, recovery, 3, RecoveryPlan::default());

    supervisor
        .replace_snapshot(
            "sim-1",
            HealthSnapshot::new()
                .with_status(HealthStatus::Critical)
                .with_failure_reasons([FailureReason::AtUnreachable]),
        )
        .await;

    supervisor.run_probe_cycle().await;
    let snapshot = supervisor
        .snapshot_for("sim-1")
        .await
        .expect("expected sim snapshot");

    assert_eq!(snapshot.current_status, HealthStatus::Healthy);
    assert!(snapshot.failure_reasons.is_empty());
    assert_eq!(snapshot.consecutive_failures, 0);
    assert!(snapshot.last_probe_at.is_some());
    assert!(snapshot.last_ok_at.is_some());
}

pub(crate) async fn assert_only_failing_sim_runs_recovery() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-a".to_string(), "sim-b".to_string()],
        [
            (
                "sim-a",
                vec![
                    HealthCheckResult::failure([FailureReason::AtUnreachable]),
                    HealthCheckResult::success(),
                ],
            ),
            ("sim-b", vec![HealthCheckResult::success()]),
        ],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let supervisor = HealthSupervisor::new(probe, recovery.clone(), 1, RecoveryPlan::default());

    supervisor.run_probe_cycle().await;

    let events = recovery.events().await;
    assert!(!events.is_empty());
    assert!(events.iter().all(|(sim_id, _)| sim_id == "sim-a"));

    let healthy = supervisor
        .snapshot_for("sim-b")
        .await
        .expect("expected healthy sim snapshot");
    assert_eq!(healthy.current_status, HealthStatus::Healthy);
    assert!(healthy.last_recovery_action.is_none());
}

pub(crate) async fn assert_latest_recovery_action_tracks_last_attempted_step() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [(
            "sim-1",
            vec![HealthCheckResult::failure([FailureReason::AtUnreachable])],
        )],
    ));
    let recovery = Arc::new(RecordingRecovery::with_failure(Some(
        RecoveryStep::SoftRestart,
    )));
    let supervisor = HealthSupervisor::new(probe, recovery, 1, RecoveryPlan::default());

    supervisor.run_probe_cycle().await;

    let snapshot = supervisor
        .snapshot_for("sim-1")
        .await
        .expect("expected sim snapshot");
    assert_eq!(snapshot.current_status, HealthStatus::Recovering);
    assert_eq!(
        snapshot.last_recovery_action,
        Some(RecoveryAction::RestartModem)
    );
}

pub(crate) async fn assert_read_sms_failed_reason_marks_snapshot_unhealthy() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [(
            "sim-1",
            vec![HealthCheckResult::failure([FailureReason::ReadSmsFailed])],
        )],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let supervisor = HealthSupervisor::new(probe, recovery, 3, RecoveryPlan::default());

    supervisor.run_probe_cycle().await;

    let snapshot = supervisor
        .snapshot_for("sim-1")
        .await
        .expect("expected sim snapshot");
    assert_eq!(snapshot.current_status, HealthStatus::Degraded);
    assert!(snapshot
        .failure_reasons
        .contains(&FailureReason::ReadSmsFailed));
}

pub(crate) async fn assert_failed_reprobe_after_recovery_becomes_critical() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [(
            "sim-1",
            vec![
                HealthCheckResult::failure([FailureReason::AtUnreachable]),
                HealthCheckResult::failure([FailureReason::ReadSmsFailed]),
            ],
        )],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let supervisor = HealthSupervisor::new(probe, recovery, 1, RecoveryPlan::default());

    supervisor.run_probe_cycle().await;

    let snapshot = supervisor
        .snapshot_for("sim-1")
        .await
        .expect("expected sim snapshot");
    assert_eq!(snapshot.current_status, HealthStatus::Critical);
    assert!(snapshot
        .failure_reasons
        .contains(&FailureReason::ReadSmsFailed));
}

pub(crate) async fn assert_other_sims_progress_while_one_sim_waits_in_recovery() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-a".to_string(), "sim-b".to_string()],
        [
            (
                "sim-a",
                vec![
                    HealthCheckResult::failure([FailureReason::AtUnreachable]),
                    HealthCheckResult::success(),
                ],
            ),
            ("sim-b", vec![HealthCheckResult::success()]),
        ],
    ));
    let recovery = Arc::new(BlockingRecovery::new("sim-a", RecoveryStep::WaitWindow));
    let supervisor = Arc::new(HealthSupervisor::new(
        probe,
        recovery.clone(),
        1,
        RecoveryPlan::default(),
    ));

    let worker = tokio::spawn({
        let supervisor = supervisor.clone();
        async move {
            supervisor.run_probe_cycle().await;
        }
    });

    let sim_b_snapshot = timeout(Duration::from_millis(100), async {
        loop {
            if let Some(snapshot) = supervisor.snapshot_for("sim-b").await {
                break snapshot;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("expected sim-b to progress before sim-a recovery wait completes");

    assert_eq!(sim_b_snapshot.current_status, HealthStatus::Healthy);

    recovery.release();
    worker.await.expect("expected supervisor worker to finish");
}

pub(crate) async fn assert_real_supervisor_path_uses_alert_gate() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [(
            "sim-1",
            vec![
                ProbeResponse::result(HealthCheckResult::failure([FailureReason::AtUnreachable])),
                ProbeResponse::result(HealthCheckResult::failure([FailureReason::AtUnreachable])),
                ProbeResponse::result(HealthCheckResult::success()),
            ],
        )],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let alerts = Arc::new(RecordingAlertSink::default());
    let supervisor = HealthSupervisor::with_alert_sink(
        probe,
        recovery,
        3,
        RecoveryPlan::default(),
        alerts.clone(),
    );

    supervisor.run_probe_cycle().await;
    supervisor.run_probe_cycle().await;
    supervisor.run_probe_cycle().await;

    let emitted = alerts.emitted().await;
    assert_eq!(emitted.len(), 2);
    assert_eq!(emitted[0].0, "sim-1");
    assert_eq!(emitted[0].1.current_status, HealthStatus::Degraded);
    assert_eq!(emitted[1].0, "sim-1");
    assert_eq!(emitted[1].1.current_status, HealthStatus::Healthy);
}

pub(crate) async fn assert_repeated_probe_errors_trigger_recovery_at_threshold() {
    let probe = Arc::new(SequenceProbe::new(
        vec!["sim-1".to_string()],
        [(
            "sim-1",
            vec![
                ProbeResponse::error("probe transport failure"),
                ProbeResponse::error("probe transport failure"),
                ProbeResponse::result(HealthCheckResult::success()),
            ],
        )],
    ));
    let recovery = Arc::new(RecordingRecovery::default());
    let supervisor = HealthSupervisor::new(probe, recovery.clone(), 2, RecoveryPlan::default());

    supervisor.run_probe_cycle().await;
    supervisor.run_probe_cycle().await;

    let snapshot = supervisor
        .snapshot_for("sim-1")
        .await
        .expect("expected sim snapshot");
    let recovery_events = recovery.events().await;

    assert!(!recovery_events.is_empty());
    assert_eq!(snapshot.current_status, HealthStatus::Healthy);
    assert_eq!(snapshot.consecutive_failures, 0);
}

#[derive(Default)]
struct RecordingRecovery {
    events: Mutex<Vec<(String, RecoveryStep)>>,
    fail_on: Option<RecoveryStep>,
}

impl RecordingRecovery {
    fn with_failure(fail_on: Option<RecoveryStep>) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            fail_on,
        }
    }

    async fn events(&self) -> Vec<(String, RecoveryStep)> {
        self.events.lock().await.clone()
    }
}

impl RecoveryExecutor for RecordingRecovery {
    fn execute_step<'a>(
        &'a self,
        sim_id: &'a str,
        step: RecoveryStep,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            self.events.lock().await.push((sim_id.to_string(), step));

            if self.fail_on == Some(step) {
                return Err(anyhow!("forced recovery failure"));
            }

            Ok(())
        })
    }
}

struct BlockingRecovery {
    blocked_sim_id: String,
    blocked_step: RecoveryStep,
    gate: Notify,
}

impl BlockingRecovery {
    fn new(blocked_sim_id: &str, blocked_step: RecoveryStep) -> Self {
        Self {
            blocked_sim_id: blocked_sim_id.to_string(),
            blocked_step,
            gate: Notify::new(),
        }
    }

    fn release(&self) {
        self.gate.notify_waiters();
    }
}

impl RecoveryExecutor for BlockingRecovery {
    fn execute_step<'a>(
        &'a self,
        sim_id: &'a str,
        step: RecoveryStep,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if sim_id == self.blocked_sim_id && step == self.blocked_step {
                self.gate.notified().await;
            }
            Ok(())
        })
    }
}

#[derive(Clone)]
enum ProbeResponse {
    Result(HealthCheckResult),
    Error(String),
}

impl ProbeResponse {
    fn result(result: HealthCheckResult) -> Self {
        Self::Result(result)
    }

    fn error(message: &str) -> Self {
        Self::Error(message.to_string())
    }
}

impl From<HealthCheckResult> for ProbeResponse {
    fn from(result: HealthCheckResult) -> Self {
        Self::Result(result)
    }
}

struct SequenceProbe {
    sim_ids: Vec<String>,
    responses: Mutex<BTreeMap<String, VecDeque<ProbeResponse>>>,
}

impl SequenceProbe {
    fn new<const N: usize, T>(sim_ids: Vec<String>, responses: [(&str, Vec<T>); N]) -> Self
    where
        T: Into<ProbeResponse>,
    {
        Self {
            sim_ids,
            responses: Mutex::new(BTreeMap::from_iter(responses.into_iter().map(
                |(sim_id, results)| {
                    (
                        sim_id.to_string(),
                        VecDeque::from(results.into_iter().map(Into::into).collect::<Vec<_>>()),
                    )
                },
            ))),
        }
    }
}

impl HealthProbe for SequenceProbe {
    fn sim_ids(&self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move { self.sim_ids.clone() })
    }

    fn run_checks<'a>(
        &'a self,
        sim_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<HealthCheckResult>> + Send + 'a>> {
        Box::pin(async move {
            let mut responses = self.responses.lock().await;
            let results = responses
                .get_mut(sim_id)
                .unwrap_or_else(|| panic!("missing probe sequence for {}", sim_id));

            match results
                .pop_front()
                .unwrap_or_else(|| ProbeResponse::result(HealthCheckResult::success()))
            {
                ProbeResponse::Result(result) => Ok(result),
                ProbeResponse::Error(message) => Err(anyhow!(message)),
            }
        })
    }
}

#[derive(Default)]
struct RecordingAlertSink {
    emitted: Mutex<Vec<(String, HealthSnapshot)>>,
}

impl RecordingAlertSink {
    async fn emitted(&self) -> Vec<(String, HealthSnapshot)> {
        self.emitted.lock().await.clone()
    }
}

impl HealthAlertSink for RecordingAlertSink {
    fn emit<'a>(
        &'a self,
        sim_id: &'a str,
        snapshot: &'a HealthSnapshot,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            self.emitted
                .lock()
                .await
                .push((sim_id.to_string(), snapshot.clone()));
        })
    }
}
