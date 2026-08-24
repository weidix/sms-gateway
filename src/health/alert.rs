use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use log::{error, warn};
use reqwest::{header::CONTENT_TYPE, Client};
use serde_json::json;
use tokio::sync::mpsc;

use crate::config::HealthWebhookConfig;
use crate::health::state::{FailureReason, HealthSnapshot, HealthStatus};
use crate::ModemManagerRef;

#[derive(Debug, Clone, Default)]
pub struct AlertGate {
    last_emitted: Option<AlertFingerprint>,
}

impl AlertGate {
    pub fn should_emit(&mut self, snapshot: &HealthSnapshot) -> bool {
        let next = AlertFingerprint::from(snapshot);
        if self.last_emitted.as_ref() == Some(&next) {
            return false;
        }

        self.last_emitted = Some(next);
        true
    }

    #[cfg(test)]
    pub fn reset(&mut self) {
        self.last_emitted = None;
    }
}

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

pub type AlertFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub trait HealthAlertSink: Send + Sync {
    fn emit<'a>(&'a self, sim_id: &'a str, snapshot: &'a HealthSnapshot) -> AlertFuture<'a>;
}

#[derive(Clone)]
pub struct HealthWebhookAlertManager {
    client: Client,
    configs: Arc<Vec<HealthWebhookConfig>>,
    modem_manager: ModemManagerRef,
    sender: mpsc::UnboundedSender<HealthAlertEvent>,
}

impl HealthWebhookAlertManager {
    pub fn new(configs: Vec<HealthWebhookConfig>, modem_manager: ModemManagerRef) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let manager = Self {
            client: Client::new(),
            configs: Arc::new(configs),
            modem_manager,
            sender,
        };

        let worker = manager.clone();
        tokio::spawn(async move {
            worker.receiver_loop(receiver).await;
        });

        manager
    }

    async fn receiver_loop(&self, mut receiver: mpsc::UnboundedReceiver<HealthAlertEvent>) {
        while let Some(event) = receiver.recv().await {
            for config in self.configs.iter() {
                self.send_webhook(config, &event).await;
            }
        }
    }

    async fn send_webhook(&self, config: &HealthWebhookConfig, event: &HealthAlertEvent) {
        let payload = build_payload(
            &event.sim_id,
            &event.com_port,
            &event.snapshot,
            event.timestamp,
        );
        let mut request = self
            .client
            .request(config.method.clone().into(), &config.url)
            .json(&payload);

        let mut has_content_type = false;
        if let Some(headers) = &config.headers {
            for (key, value) in headers {
                has_content_type |= key.eq_ignore_ascii_case(CONTENT_TYPE.as_str());
                request = request.header(key, value);
            }
        }
        if !has_content_type {
            request = request.header(CONTENT_TYPE, "application/json");
        }
        if let Some(timeout_seconds) = config.timeout {
            request = request.timeout(Duration::from_secs(timeout_seconds));
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => {
                warn!(
                    "Health webhook returned non-success status {} for sim {}",
                    response.status(),
                    event.sim_id
                );
            }
            Err(err) => {
                error!(
                    "Failed to send health webhook for sim {}: {}",
                    event.sim_id, err
                );
            }
        }
    }
}

impl HealthAlertSink for HealthWebhookAlertManager {
    fn emit<'a>(&'a self, sim_id: &'a str, snapshot: &'a HealthSnapshot) -> AlertFuture<'a> {
        Box::pin(async move {
            let com_port = self
                .modem_manager
                .get_modem(sim_id)
                .await
                .map(|modem| modem.com_port.clone())
                .unwrap_or_default();

            if let Err(err) = self.sender.send(HealthAlertEvent {
                sim_id: sim_id.to_string(),
                com_port,
                snapshot: snapshot.clone(),
                timestamp: Utc::now(),
            }) {
                error!(
                    "Failed to enqueue health webhook alert for sim {}: {}",
                    sim_id, err
                );
            }
        })
    }
}

#[derive(Debug, Clone)]
struct HealthAlertEvent {
    sim_id: String,
    com_port: String,
    snapshot: HealthSnapshot,
    timestamp: DateTime<Utc>,
}

fn build_payload(
    sim_id: &str,
    com_port: &str,
    snapshot: &HealthSnapshot,
    timestamp: DateTime<Utc>,
) -> serde_json::Value {
    json!({
        "sim_id": sim_id,
        "com_port": com_port,
        "status": snapshot.current_status.as_str(),
        "failure_reasons": snapshot
            .failure_reasons
            .iter()
            .copied()
            .map(FailureReason::as_str)
            .collect::<Vec<_>>(),
        "consecutive_failures": snapshot.consecutive_failures,
        "timestamp": timestamp.to_rfc3339(),
        "last_probe_at": snapshot.last_probe_at.map(|value| value.to_rfc3339()),
        "last_ok_at": snapshot.last_ok_at.map(|value| value.to_rfc3339()),
        "last_recovery_action": snapshot.last_recovery_action.map(|action| action.as_str()),
        "last_recovery_at": snapshot.last_recovery_at.map(|value| value.to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::Value;

    #[test]
    fn duplicate_status_and_reason_set_does_not_emit_alert() {
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

    #[test]
    fn payload_contains_minimum_contract_fields() {
        let timestamp = Utc.with_ymd_and_hms(2026, 5, 4, 12, 30, 45).unwrap();
        let snapshot = HealthSnapshot::new()
            .record_failure(3, [FailureReason::AtUnreachable], None)
            .with_status(HealthStatus::Recovering);

        let payload = build_payload("sim-1", "/dev/ttyUSB0", &snapshot, timestamp);

        assert_eq!(
            payload.get("sim_id"),
            Some(&Value::String("sim-1".to_string()))
        );
        assert_eq!(
            payload.get("com_port"),
            Some(&Value::String("/dev/ttyUSB0".to_string()))
        );
        assert_eq!(
            payload.get("status"),
            Some(&Value::String("recovering".to_string()))
        );
        assert_eq!(
            payload.get("timestamp"),
            Some(&Value::String(timestamp.to_rfc3339()))
        );
        assert_eq!(
            payload.get("consecutive_failures"),
            Some(&Value::Number(1_u64.into()))
        );
        assert!(payload.get("failure_reasons").is_some());
        assert!(payload.get("last_recovery_action").is_some());
        assert!(payload.get("current_status").is_none());
    }
}
