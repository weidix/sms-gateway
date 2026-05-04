use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use api::SseManager;
use config::Settings;
use db::db_init;
use flexi_logger::{
    colored_detailed_format, Age, Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming,
};
use log::LevelFilter;
use modem::{ModemManager, SmsType};
use structopt::StructOpt;

mod api;
mod config;
mod db;
mod decode;
mod health;
mod modem;
#[cfg(test)]
mod tests;
mod update;
mod webhook;

pub type ModemManagerRef = Arc<ModemManager>;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let param = Param::from_args();
    if let Some(command) = param.command {
        match command {
            Command::Update => {
                update::run_update().await.context("Update failed")?;
            }
            Command::Version => {
                println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            }
        }
        return Ok(());
    }
    log_init(&param.log_path, &param.log_level)?;
    db_init().await?;
    #[cfg(debug_assertions)]
    let config = config::AppConfig::load(&PathBuf::from("./config.toml"))?;
    #[cfg(not(debug_assertions))]
    let config = config::AppConfig::load(&param.config_file)?;

    let modem_manager = Arc::new(
        ModemManager::initialize(&config)
            .await
            .context("Failed to initialize ModemManager")?,
    );

    let sse_manager = Arc::new(api::SseManager::new());

    let webhook_manager = match config.settings.webhooks.clone() {
        Some(cfgs) => Some(webhook::start_webhook_worker_with_concurrency(
            cfgs,
            config.settings.webhooks_max_concurrent.unwrap_or(1),
        )),
        _ => None,
    };

    tokio::spawn(read_sms_worker(
        modem_manager.clone(),
        config.settings.read_sms_frequency,
        sse_manager.clone(),
        webhook_manager,
    ));
    let _health_supervisor = health::start_supervisor(modem_manager.clone(), &config.settings);

    run_api_for_settings(modem_manager, &config.settings, sse_manager).await
}

async fn read_sms_worker(
    modem_manager: ModemManagerRef,
    read_sms_frequency: u64,
    sse_manager: Arc<SseManager>,
    webhook_manager: Option<webhook::WebhookManager>,
) {
    loop {
        modem_manager
            .read_all_sms_async(
                SmsType::RecUnread,
                sse_manager.clone(),
                webhook_manager.clone(),
            )
            .await;

        tokio::time::sleep(tokio::time::Duration::from_secs(read_sms_frequency)).await;
    }
}

#[derive(Debug, StructOpt)]
pub struct Param {
    #[structopt(subcommand)]
    pub command: Option<Command>,

    #[cfg(debug_assertions)]
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "./logs"
    )]
    pub log_path: PathBuf,

    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "l",
        long = "log",
        parse(from_os_str),
        default_value = "/var/lib/sms-gateway/log"
    )]
    pub log_path: PathBuf,

    #[cfg(debug_assertions)]
    #[structopt(
        short = "v",
        long = "log-level",
        default_value = "debug",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: LevelFilter,

    #[cfg(not(debug_assertions))]
    #[structopt(
        short = "v",
        long = "log-level",
        default_value = "info",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: LevelFilter,

    #[structopt(
        short = "c",
        long = "config",
        parse(from_os_str),
        default_value = "/etc/sms-gateway/config.toml"
    )]
    pub config_file: PathBuf,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Update sms-gateway to the latest release
    Update,
    /// Show version information
    Version,
}

fn log_init(log_path: &PathBuf, log_level: &LevelFilter) -> anyhow::Result<()> {
    if !log_path.exists() {
        std::fs::create_dir_all(log_path)?;
    }
    let file_spec = FileSpec::default().directory(log_path);

    let _ = Logger::try_with_str(format!("{}", log_level))?
        .log_to_file(file_spec)
        .duplicate_to_stderr(Duplicate::All)
        .format_for_stderr(colored_detailed_format)
        .format_for_stdout(colored_detailed_format)
        //https://upload.wikimedia.org/wikipedia/commons/1/15/Xterm_256color_chart.svg
        .set_palette(String::from("b196;208;28;7;8"))
        .rotate(
            Criterion::Age(Age::Day),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(7),
        )
        .start()?;
    Ok(())
}

// SIM检测逻辑已完全移除 - 设备映射在启动时建立，运行时不再检测

fn resolve_basic_auth(settings: &Settings) -> anyhow::Result<Option<(String, String)>> {
    match (&settings.username, &settings.password) {
        (Some(username), Some(password)) => Ok(Some((username.clone(), password.clone()))),
        (None, None) => Ok(None),
        _ => Err(anyhow::anyhow!(
            "Both username and password must be set together to enable authentication"
        )),
    }
}

async fn invoke_api_runner<F, Fut>(
    settings: &Settings,
    sse_manager: Arc<SseManager>,
    runner: F,
) -> anyhow::Result<()>
where
    F: FnOnce(String, u16, Option<(String, String)>, Arc<SseManager>) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    let auth = resolve_basic_auth(settings)?;
    runner(
        settings.server_host.clone(),
        settings.server_port,
        auth,
        sse_manager,
    )
    .await
}

async fn run_api_for_settings(
    modem_manager: ModemManagerRef,
    settings: &Settings,
    sse_manager: Arc<SseManager>,
) -> anyhow::Result<()> {
    invoke_api_runner(
        settings,
        sse_manager,
        move |host, port, auth, sse_manager| async move {
            api::run_api(
                modem_manager,
                &host,
                &port,
                auth.as_ref()
                    .map(|(username, password)| (username.as_str(), password.as_str())),
                sse_manager,
            )
            .await
        },
    )
    .await
}

#[cfg(test)]
mod main_tests {
    use super::*;

    fn test_settings(username: Option<&str>, password: Option<&str>) -> config::Settings {
        config::Settings {
            server_host: "127.0.0.1".to_string(),
            server_port: 0,
            username: username.map(str::to_string),
            password: password.map(str::to_string),
            read_sms_frequency: 30,
            health_check_frequency: 30,
            health_failure_threshold: 3,
            health_restart_wait_seconds: 20,
            webhooks_max_concurrent: None,
            webhooks: None,
            health_webhooks: None,
            sms_storage: None,
        }
    }

    #[test]
    fn resolve_basic_auth_allows_missing_credentials() {
        let auth = resolve_basic_auth(&test_settings(None, None)).unwrap();
        assert!(auth.is_none());
    }

    #[test]
    fn resolve_basic_auth_rejects_partial_credentials() {
        let err = resolve_basic_auth(&test_settings(Some("admin"), None)).unwrap_err();
        assert!(err.to_string().contains("username and password"));
    }

    #[tokio::test]
    async fn invoke_api_runner_propagates_errors() {
        let settings = test_settings(Some("admin"), Some("secret"));
        let sse_manager = Arc::new(api::SseManager::new());

        let err = invoke_api_runner(
            &settings,
            sse_manager,
            |host, port, auth, _sse_manager| async move {
                assert_eq!(host, "127.0.0.1");
                assert_eq!(port, 0);
                assert_eq!(auth, Some(("admin".to_string(), "secret".to_string())));
                Err(anyhow::anyhow!("bind failed"))
            },
        )
        .await
        .unwrap_err();

        assert!(err.to_string().contains("bind failed"));
    }
}

#[cfg(test)]
#[test]
fn parses_sms_storage_status_from_cpms() {
    crate::modem::types::assert_parses_sms_storage_status_from_cpms();
}

#[cfg(test)]
#[test]
fn network_registration_treats_home_and_roaming_as_registered() {
    crate::modem::types::assert_network_registration_treats_home_and_roaming_as_registered();
}

#[cfg(test)]
#[test]
fn deserializes_health_settings_and_webhooks() {
    crate::config::assert_deserializes_health_settings_and_webhooks();
}

#[cfg(test)]
#[test]
fn rejects_zero_health_check_frequency() {
    crate::config::assert_rejects_zero_health_check_frequency();
}

#[cfg(test)]
#[test]
fn rejects_zero_health_failure_threshold() {
    crate::config::assert_rejects_zero_health_failure_threshold();
}

#[cfg(test)]
#[test]
fn rejects_zero_health_restart_wait_seconds() {
    crate::config::assert_rejects_zero_health_restart_wait_seconds();
}

#[cfg(test)]
#[test]
fn rejects_invalid_health_webhook_config() {
    crate::config::assert_rejects_invalid_health_webhook_config();
}

#[cfg(test)]
#[test]
fn state_machine_moves_to_recovering_at_threshold() {
    crate::health::state::assert_state_machine_moves_to_recovering_at_threshold();
}

#[cfg(test)]
#[test]
fn duplicate_status_and_reason_set_does_not_emit_alert() {
    crate::health::alert::assert_duplicate_status_and_reason_set_does_not_emit_alert();
}

#[cfg(test)]
#[test]
fn default_recovery_plan_is_stable() {
    crate::tests::health_tests::assert_default_recovery_plan_is_stable();
}

#[cfg(test)]
#[tokio::test]
async fn unhealthy_snapshot_returns_to_healthy_after_probe_success() {
    crate::tests::health_tests::assert_unhealthy_snapshot_returns_to_healthy_after_probe_success()
        .await;
}

#[cfg(test)]
#[tokio::test]
async fn only_failing_sim_runs_recovery() {
    crate::tests::health_tests::assert_only_failing_sim_runs_recovery().await;
}

#[cfg(test)]
#[tokio::test]
async fn latest_recovery_action_tracks_last_attempted_step() {
    crate::tests::health_tests::assert_latest_recovery_action_tracks_last_attempted_step().await;
}

#[cfg(test)]
#[tokio::test]
async fn read_sms_failed_reason_marks_snapshot_unhealthy() {
    crate::tests::health_tests::assert_read_sms_failed_reason_marks_snapshot_unhealthy().await;
}

#[cfg(test)]
#[tokio::test]
async fn failed_reprobe_after_recovery_becomes_critical() {
    crate::tests::health_tests::assert_failed_reprobe_after_recovery_becomes_critical().await;
}

#[cfg(test)]
#[test]
fn d_probe_uses_configured_receive_storage_semantics() {
    crate::health::probe::assert_d_probe_uses_configured_receive_storage_semantics();
}

#[cfg(test)]
#[tokio::test]
async fn other_sims_progress_while_one_sim_waits_in_recovery() {
    crate::tests::health_tests::assert_other_sims_progress_while_one_sim_waits_in_recovery().await;
}
