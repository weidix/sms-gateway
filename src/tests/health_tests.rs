use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::health::probe::{HealthCheckResult, HealthProbe};
use crate::health::recovery::{RecoveryExecutor, RecoveryPlan, RecoveryStep};
use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus};
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
    let probe = Arc::new(SequenceProbe::new(vec![HealthCheckResult::success()]));
    let recovery = Arc::new(NoopRecovery);
    let supervisor = HealthSupervisor::new(probe, recovery, 3, RecoveryPlan::default());

    supervisor
        .replace_snapshot(
            HealthSnapshot::new()
                .with_status(HealthStatus::Critical)
                .with_failure_reasons([FailureReason::AtUnreachable]),
        )
        .await;

    let snapshot = supervisor.run_probe_cycle().await;

    assert_eq!(snapshot.current_status, HealthStatus::Healthy);
    assert!(snapshot.failure_reasons.is_empty());
    assert_eq!(snapshot.consecutive_failures, 0);
    assert!(snapshot.last_probe_at.is_some());
    assert!(snapshot.last_ok_at.is_some());
}

struct NoopRecovery;

impl RecoveryExecutor for NoopRecovery {
    fn execute<'a>(
        &'a self,
        _plan: &'a RecoveryPlan,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
}

struct SequenceProbe {
    responses: Mutex<VecDeque<HealthCheckResult>>,
}

impl SequenceProbe {
    fn new(responses: Vec<HealthCheckResult>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(responses)),
        }
    }
}

impl HealthProbe for SequenceProbe {
    fn run_checks(&self) -> Pin<Box<dyn Future<Output = Result<HealthCheckResult>> + Send + '_>> {
        Box::pin(async move {
            let mut responses = self.responses.lock().await;
            Ok(responses
                .pop_front()
                .unwrap_or_else(HealthCheckResult::success))
        })
    }
}
