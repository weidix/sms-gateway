pub mod alert;
pub mod probe;
pub mod recovery;
pub mod state;
pub mod supervisor;

use std::{sync::Arc, time::Duration};

use crate::{config::Settings, ModemManagerRef};

use probe::ModemHealthProbe;
use recovery::{ModemRecovery, RecoveryPlan};
use supervisor::HealthSupervisor;

pub fn start_supervisor(
    modem_manager: ModemManagerRef,
    settings: &Settings,
) -> Arc<HealthSupervisor> {
    let probe = Arc::new(ModemHealthProbe::new(modem_manager.clone()));
    let recovery = Arc::new(ModemRecovery::new(
        modem_manager,
        settings.sms_storage,
        Duration::from_secs(settings.health_restart_wait_seconds),
    ));
    let supervisor = Arc::new(HealthSupervisor::new(
        probe,
        recovery,
        settings.health_failure_threshold,
        RecoveryPlan::default(),
    ));

    supervisor
        .clone()
        .spawn_worker(Duration::from_secs(settings.health_check_frequency));

    supervisor
}
