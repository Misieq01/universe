use std::time::{Duration, SystemTime};

use crate::{
    progress_trackers::{progress_stepper::ProgressStepperBuilder, ProgressStepper},
    tasks_tracker::TasksTracker,
};
use anyhow::Error;
use log::{error, info};
use tauri::AppHandle;
use tauri_plugin_sentry::sentry;

use super::{
    setup_manager::{SetupManager, SetupPhase},
    trait_setup_phase::SetupPhaseImpl,
};

static LOG_TARGET: &str = "tari::universe::phase_hardware";
const SETUP_TIMEOUT_DURATION: Duration = Duration::from_secs(60 * 10); // 10 Minutes
#[derive(Clone, Default)]
pub struct HardwareSetupPhaseSessionConfiguration {}

#[derive(Clone, Default)]
pub struct HardwareSetupPhaseAppConfiguration {
    is_auto_launcher_enabled: bool,
    last_binaries_update_timestamp: Option<SystemTime>,
    use_tor: bool,
}

pub struct CoreSetupPhase {
    progress_stepper: ProgressStepper,
    app_configuration: HardwareSetupPhaseAppConfiguration,
    session_configuration: HardwareSetupPhaseSessionConfiguration,
}

impl SetupPhaseImpl for CoreSetupPhase {
    type Configuration = HardwareSetupPhaseAppConfiguration;

    fn new() -> Self {
        CoreSetupPhase {
            progress_stepper: Self::create_progress_stepper(),
            app_configuration: HardwareSetupPhaseAppConfiguration::default(),
            session_configuration: HardwareSetupPhaseSessionConfiguration::default(),
        }
    }

    fn create_progress_stepper() -> ProgressStepper {
        let progress_stepper = ProgressStepperBuilder::new().build();

        progress_stepper
    }

    async fn load_configuration(
        &mut self,
        configuration: Self::Configuration,
    ) -> Result<(), Error> {
        self.app_configuration = configuration;

        Ok(())
    }

    async fn setup(self: std::sync::Arc<Self>, app_handle: AppHandle) {
        info!(target: LOG_TARGET, "[ Hardware Phase ] Starting setup");

        TasksTracker::current().spawn(async move {
            let setup_timeout = tokio::time::sleep(SETUP_TIMEOUT_DURATION);
            tokio::select! {
                _ = setup_timeout => {
                    error!(target: LOG_TARGET, "[ Hardware Phase ] Setup timed out");
                    let error_message = "[ Hardware Phase ] Setup timed out";
                    sentry::capture_message(&error_message, sentry::Level::Error);
                }
                result = self.setup_inner(app_handle.clone()) => {
                    match result {
                        Ok(_) => {
                            info!(target: LOG_TARGET, "[ Hardware Phase ] Setup completed successfully");
                            self.finalize_setup(app_handle.clone()).await();
                        }
                        Err(error) => {
                            error!(target: LOG_TARGET, "[ Hardware Phase ] Setup failed with error: {:?}", error);
                            let error_message = format!("[ Hardware Phase ] Setup failed with error: {:?}", error);
                            sentry::capture_message(&error_message, sentry::Level::Error);
                        }
                    }
                }
            };
        });
    }

    async fn setup_inner(&self, app_handle: AppHandle) -> Result<(), Error> {
        todo!()
    }

    async fn finalize_setup(&self, app_handle: AppHandle) -> Result<(), Error> {
        SetupManager::get_instance()
            .lock()
            .await
            .set_phase_status(SetupPhase::Hardware, true);

        // Todo: send event
        Ok(())
    }
}
