use std::collections::BTreeSet;

use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus};

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AlertGate {
    last_emitted: Option<AlertFingerprint>,
}

#[allow(dead_code)]
impl AlertGate {
    pub fn should_emit(&mut self, snapshot: &HealthSnapshot) -> bool {
        let next = AlertFingerprint::from(snapshot);
        if self.last_emitted.as_ref() == Some(&next) {
            return false;
        }

        self.last_emitted = Some(next);
        true
    }

    pub fn reset(&mut self) {
        self.last_emitted = None;
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct AlertFingerprint {
    status: HealthStatus,
    failure_reasons: BTreeSet<FailureReason>,
}

impl From<&HealthSnapshot> for AlertFingerprint {
    fn from(snapshot: &HealthSnapshot) -> Self {
        Self {
            status: snapshot.current_status,
            failure_reasons: snapshot.failure_reasons.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn duplicate_status_and_reason_set_does_not_emit_alert() {
        let first = HealthSnapshot::new().record_failure(3, [FailureReason::AtUnreachable], None);
        let duplicate =
            HealthSnapshot::new().record_failure(3, [FailureReason::AtUnreachable], None);

        let mut gate = AlertGate::default();

        assert!(gate.should_emit(&first));
        assert!(!gate.should_emit(&duplicate));

        let changed_status = HealthSnapshot::new()
            .with_status(HealthStatus::Critical)
            .with_failure_reasons([FailureReason::AtUnreachable]);
        assert!(gate.should_emit(&changed_status));
    }

    #[test]
    fn reset_allows_same_fingerprint_to_emit_again() {
        let unhealthy =
            HealthSnapshot::new().record_failure(3, [FailureReason::AtUnreachable], None);
        let healthy = HealthSnapshot::new().record_success();
        let mut gate = AlertGate::default();

        assert!(gate.should_emit(&unhealthy));
        assert!(!gate.should_emit(&unhealthy));

        gate.reset();
        assert!(gate.should_emit(&unhealthy));

        assert!(gate.should_emit(&healthy));
        gate.reset();
        assert!(gate.should_emit(&unhealthy));
    }
}

#[cfg(test)]
pub(crate) fn assert_duplicate_status_and_reason_set_does_not_emit_alert() {
    tests::duplicate_status_and_reason_set_does_not_emit_alert();
}
