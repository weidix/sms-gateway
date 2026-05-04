use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Recovering,
    Critical,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FailureReason {
    AtUnreachable,
    SimNotReady,
    NetworkNotRegistered,
    SmsStorageUnavailable,
    SmsStorageFull,
    ReadSmsFailed,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    ReinitializeModem,
    ReapplySmsStorage,
    RestartModem,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub current_status: HealthStatus,
    pub failure_reasons: BTreeSet<FailureReason>,
    pub consecutive_failures: u64,
    pub last_probe_at: Option<DateTime<Utc>>,
    pub last_ok_at: Option<DateTime<Utc>>,
    pub last_recovery_action: Option<RecoveryAction>,
    pub last_recovery_at: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
impl HealthSnapshot {
    pub fn new() -> Self {
        Self {
            current_status: HealthStatus::Healthy,
            failure_reasons: BTreeSet::new(),
            consecutive_failures: 0,
            last_probe_at: None,
            last_ok_at: None,
            last_recovery_action: None,
            last_recovery_at: None,
        }
    }

    pub fn record_success(&self) -> Self {
        let now = Utc::now();

        Self {
            current_status: HealthStatus::Healthy,
            failure_reasons: BTreeSet::new(),
            consecutive_failures: 0,
            last_probe_at: Some(now),
            last_ok_at: Some(now),
            last_recovery_action: self.last_recovery_action,
            last_recovery_at: self.last_recovery_at,
        }
    }

    pub fn record_failure<I>(
        &self,
        failure_threshold: u64,
        reasons: I,
        recovery_action: Option<RecoveryAction>,
    ) -> Self
    where
        I: IntoIterator<Item = FailureReason>,
    {
        let now = Utc::now();
        let consecutive_failures = self.consecutive_failures + 1;
        let failure_reasons = reasons.into_iter().collect::<BTreeSet<_>>();
        let reached_threshold = consecutive_failures >= failure_threshold;
        let current_status = if reached_threshold {
            HealthStatus::Recovering
        } else {
            HealthStatus::Degraded
        };
        let latest_recovery_action = if reached_threshold {
            recovery_action.or(self.last_recovery_action)
        } else {
            self.last_recovery_action
        };

        Self {
            current_status,
            failure_reasons,
            consecutive_failures,
            last_probe_at: Some(now),
            last_ok_at: self.last_ok_at,
            last_recovery_action: latest_recovery_action,
            last_recovery_at: if reached_threshold && latest_recovery_action.is_some() {
                Some(now)
            } else {
                self.last_recovery_at
            },
        }
    }

    pub fn record_recovery_action(&self, action: RecoveryAction) -> Self {
        Self {
            current_status: HealthStatus::Recovering,
            failure_reasons: self.failure_reasons.clone(),
            consecutive_failures: self.consecutive_failures,
            last_probe_at: self.last_probe_at,
            last_ok_at: self.last_ok_at,
            last_recovery_action: Some(action),
            last_recovery_at: Some(Utc::now()),
        }
    }

    pub fn with_status(mut self, status: HealthStatus) -> Self {
        self.current_status = status;
        self
    }

    pub fn with_failure_reasons<I>(mut self, reasons: I) -> Self
    where
        I: IntoIterator<Item = FailureReason>,
    {
        self.failure_reasons = reasons.into_iter().collect();
        self
    }
}

impl Default for HealthSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) fn assert_state_machine_moves_to_recovering_at_threshold() {
    tests::state_machine_moves_to_recovering_at_threshold();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    pub(crate) fn state_machine_moves_to_recovering_at_threshold() {
        let snapshot = HealthSnapshot {
            consecutive_failures: 2,
            ..HealthSnapshot::new()
        };
        let failed = snapshot.record_failure(
            3,
            [FailureReason::AtUnreachable, FailureReason::SimNotReady],
            Some(RecoveryAction::ReinitializeModem),
        );

        assert_eq!(failed.current_status, HealthStatus::Recovering);
        assert_eq!(failed.consecutive_failures, 3);
        assert_eq!(
            failed.failure_reasons,
            std::collections::BTreeSet::from([
                FailureReason::AtUnreachable,
                FailureReason::SimNotReady,
            ])
        );
        assert_eq!(
            failed.last_recovery_action,
            Some(RecoveryAction::ReinitializeModem)
        );
        assert!(failed.last_probe_at.is_some());
        assert!(failed.last_recovery_at.is_some());
        assert!(failed.last_ok_at.is_none());
    }

    #[test]
    fn repeated_failure_while_recovering_updates_latest_recovery_action() {
        let original_recovery_at = Utc.with_ymd_and_hms(2026, 5, 4, 10, 0, 0).unwrap();
        let snapshot = HealthSnapshot {
            current_status: HealthStatus::Recovering,
            consecutive_failures: 3,
            last_recovery_action: Some(RecoveryAction::ReinitializeModem),
            last_recovery_at: Some(original_recovery_at),
            ..HealthSnapshot::new()
        };

        let failed = snapshot.record_failure(
            3,
            [FailureReason::AtUnreachable],
            Some(RecoveryAction::RestartModem),
        );

        assert_eq!(failed.current_status, HealthStatus::Recovering);
        assert_eq!(failed.consecutive_failures, 4);
        assert_eq!(
            failed.last_recovery_action,
            Some(RecoveryAction::RestartModem)
        );
        assert_ne!(failed.last_recovery_at, Some(original_recovery_at));
    }
}
