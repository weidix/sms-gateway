use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;

use crate::ModemManagerRef;

use super::{probe::HealthFuture, state::RecoveryAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStep {
    ReinitializeRuntime,
    ReapplyStorageIfConfigured,
    SoftRestart,
    WaitWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPlan {
    steps: Vec<RecoveryStep>,
}

impl RecoveryPlan {
    pub fn steps(&self) -> &[RecoveryStep] {
        &self.steps
    }
}

impl Default for RecoveryPlan {
    fn default() -> Self {
        Self {
            steps: vec![
                RecoveryStep::ReinitializeRuntime,
                RecoveryStep::ReapplyStorageIfConfigured,
                RecoveryStep::SoftRestart,
                RecoveryStep::WaitWindow,
                RecoveryStep::ReinitializeRuntime,
            ],
        }
    }
}

pub trait RecoveryExecutor: Send + Sync {
    fn execute_step<'a>(
        &'a self,
        sim_id: &'a str,
        step: RecoveryStep,
    ) -> HealthFuture<'a, Result<()>>;
}

pub struct ModemRecovery {
    modem_manager: ModemManagerRef,
    wait_window: Duration,
}

impl ModemRecovery {
    pub fn new(modem_manager: ModemManagerRef, wait_window: Duration) -> Self {
        Self {
            modem_manager,
            wait_window,
        }
    }
}

impl RecoveryStep {
    pub fn recovery_action(self) -> Option<RecoveryAction> {
        match self {
            RecoveryStep::ReinitializeRuntime => Some(RecoveryAction::ReinitializeModem),
            RecoveryStep::ReapplyStorageIfConfigured => Some(RecoveryAction::ReapplySmsStorage),
            RecoveryStep::SoftRestart => Some(RecoveryAction::RestartModem),
            RecoveryStep::WaitWindow => None,
        }
    }
}

impl RecoveryExecutor for ModemRecovery {
    fn execute_step<'a>(
        &'a self,
        sim_id: &'a str,
        step: RecoveryStep,
    ) -> HealthFuture<'a, Result<()>> {
        Box::pin(async move {
            match step {
                RecoveryStep::ReinitializeRuntime => self
                    .modem_manager
                    .reinitialize_runtime(sim_id)
                    .await
                    .with_context(|| {
                        format!("Failed to reinitialize modem runtime for {}", sim_id)
                    })?,
                RecoveryStep::ReapplyStorageIfConfigured => self
                    .modem_manager
                    .reapply_configured_sms_storage(sim_id)
                    .await
                    .with_context(|| format!("Failed to reapply SMS storage for {}", sim_id))?,
                RecoveryStep::SoftRestart => self
                    .modem_manager
                    .soft_restart(sim_id)
                    .await
                    .with_context(|| format!("Failed to soft restart modem {}", sim_id))?,
                RecoveryStep::WaitWindow => sleep(self.wait_window).await,
            }

            Ok(())
        })
    }
}
