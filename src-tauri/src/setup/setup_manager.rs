use anyhow::Error;
use getset::{Getters, Setters};
use log::info;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};
use tauri::{AppHandle, Manager};
use tokio::sync::{
    watch::{channel, Receiver, Sender},
    Mutex,
};

use crate::{events::SetupStatusPayload, initialize_frontend_updates, UniverseAppState};

use super::{
    phase_core::{CoreSetupPhase, CoreSetupPhaseSessionConfiguration},
    phase_hardware::{
        HardwareSetupPhase, HardwareSetupPhasePayload, HardwareSetupPhaseSessionConfiguration,
    },
    phase_local_node::{LocalNodeSetupPhase, LocalNodeSetupPhaseSessionConfiguration},
    phase_remote_node::{RemoteNodeSetupPhase, RemoteNodeSetupPhaseSessionConfiguration},
    phase_unknown::{UnknownSetupPhase, UnknownSetupPhaseSessionConfiguration},
    phase_wallet::{WalletSetupPhase, WalletSetupPhaseSessionConfiguration},
    trait_setup_phase::SetupPhaseImpl,
};

static LOG_TARGET: &str = "tari::universe::setup_manager";

static INSTANCE: LazyLock<Mutex<SetupManager>> = LazyLock::new(|| Mutex::new(SetupManager::new()));

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SetupPhase {
    Core,
    Wallet,
    Hardware,
    LocalNode,
    RemoteNode,
    Unknown,
}

#[derive(Getters, Setters)]

pub struct SetupManager {
    phase_statuses: HashMap<SetupPhase, bool>,
    #[getset(get = "pub", set = "pub")]
    hardware_status_output: Option<HardwareSetupPhasePayload>,
}

impl SetupManager {
    pub fn new() -> Self {
        let mut phase_statuses: HashMap<SetupPhase, bool> = HashMap::new();
        phase_statuses.insert(SetupPhase::Core, false);
        phase_statuses.insert(SetupPhase::Wallet, false);
        phase_statuses.insert(SetupPhase::Hardware, false);
        phase_statuses.insert(SetupPhase::LocalNode, false);
        phase_statuses.insert(SetupPhase::RemoteNode, false);
        Self {
            phase_statuses,
            hardware_status_output: None,
        }
    }

    pub fn get_instance() -> &'static LazyLock<Mutex<SetupManager>> {
        &INSTANCE
    }

    pub async fn start_setup(&self, app_handle: AppHandle) {
        let mut core_phase_setup = CoreSetupPhase::new();
        core_phase_setup
            .load_configuration(CoreSetupPhaseSessionConfiguration {})
            .await;
        let core_phase_setup = Arc::new(core_phase_setup);
        core_phase_setup.setup(app_handle.clone()).await;
    }

    pub async fn spawn_first_batch_of_setup_phases(&self, app_handle: AppHandle) {
        let mut hardware_phase_setup = HardwareSetupPhase::new();
        hardware_phase_setup
            .load_configuration(HardwareSetupPhaseSessionConfiguration {})
            .await;
        let hardware_phase_setup = Arc::new(hardware_phase_setup);
        hardware_phase_setup.setup(app_handle.clone()).await;

        let mut local_node_phase_setup = LocalNodeSetupPhase::new();
        local_node_phase_setup
            .load_configuration(LocalNodeSetupPhaseSessionConfiguration {})
            .await;
        let local_node_phase_setup = Arc::new(local_node_phase_setup);
        local_node_phase_setup.setup(app_handle.clone()).await;

        // let mut remote_node_phase_setup = RemoteNodeSetupPhase::new();
        // remote_node_phase_setup
        //     .load_configuration(RemoteNodeSetupPhaseSessionConfiguration {})
        //     .await;
        // let remote_node_phase_setup = Arc::new(remote_node_phase_setup);
        // remote_node_phase_setup.setup(app_handle.clone()).await;
    }

    pub async fn spawn_second_batch_of_setup_phases(&self, app_handle: AppHandle) {
        let mut wallet_phase_setup = WalletSetupPhase::new();
        wallet_phase_setup
            .load_configuration(WalletSetupPhaseSessionConfiguration {})
            .await;
        let wallet_phase_setup = Arc::new(wallet_phase_setup);
        wallet_phase_setup.setup(app_handle.clone()).await;

        let mut unknown_phase_setup = UnknownSetupPhase::new();
        let cpu_benchmarked_hashrate = self
            .hardware_status_output
            .clone()
            .unwrap_or_default()
            .cpu_benchmarked_hashrate;
        unknown_phase_setup
            .load_configuration(UnknownSetupPhaseSessionConfiguration {
                cpu_benchmarked_hashrate,
            })
            .await;
        let unknown_phase_setup = Arc::new(unknown_phase_setup);
        unknown_phase_setup.setup(app_handle.clone()).await;
    }

    pub async fn set_phase_status(
        &mut self,
        app_handle: AppHandle,
        phase: SetupPhase,
        status: bool,
    ) {
        self.phase_statuses.insert(phase, status);

        // //Todo: handle phase status update
        // self.handle_phase_status_update(app_handle, &self.phase_statuses)
        //     .await;
        let core_phase_status = *self.phase_statuses.get(&SetupPhase::Core).unwrap_or(&false);
        if core_phase_status {
            self.spawn_first_batch_of_setup_phases(app_handle.clone())
                .await;
        };
    }

    pub async fn set_phase_status_first(
        &mut self,
        app_handle: AppHandle,
        phase: SetupPhase,
        status: bool,
    ) {
        self.phase_statuses.insert(phase, status);

        // //Todo: handle phase status update
        // self.handle_phase_status_update(app_handle, &self.phase_statuses)
        //     .await;

        let hardware_phase_status = *self
            .phase_statuses
            .get(&SetupPhase::Hardware)
            .unwrap_or(&false);
        let local_node_phase_status = *self
            .phase_statuses
            .get(&SetupPhase::LocalNode)
            .unwrap_or(&false);
        let remote_node_phase_status = *self
            .phase_statuses
            .get(&SetupPhase::RemoteNode)
            .unwrap_or(&false);

        if hardware_phase_status && (local_node_phase_status || remote_node_phase_status) {
            self.spawn_second_batch_of_setup_phases(app_handle.clone())
                .await;
        }
    }

    pub async fn set_phase_status_second(
        &mut self,
        app_handle: AppHandle,
        phase: SetupPhase,
        status: bool,
    ) {
        self.phase_statuses.insert(phase, status);

        // //Todo: handle phase status update
        // self.handle_phase_status_update(app_handle, &self.phase_statuses)
        //     .await;

        let wallet_phase_status = *self
            .phase_statuses
            .get(&SetupPhase::Wallet)
            .unwrap_or(&false);
        let unknown_phase_status = *self
            .phase_statuses
            .get(&SetupPhase::Unknown)
            .unwrap_or(&false);

        if wallet_phase_status && unknown_phase_status {
            let state = app_handle.state::<UniverseAppState>();
            initialize_frontend_updates(&app_handle).await;
            state
                .events_manager
                .handle_setup_status(
                    &app_handle,
                    SetupStatusPayload {
                        event_type: "setup_status".to_string(),
                        title: "application-started".to_string(),
                        title_params: None,
                        progress: 1.0,
                    },
                )
                .await;
        }
    }

    fn unlock_app(&self) {
        todo!()
    }

    fn unlock_wallet(&self) {
        todo!()
    }

    fn unlock_mining(&self) {
        todo!()
    }

    // async fn handle_phase_status_update(
    //     &self,
    //     app_handle: AppHandle,
    //     phases_statuses: &HashMap<SetupPhase, bool>,
    // ) -> Result<(), Error> {
    //     let core_phase_status = *phases_statuses.get(&SetupPhase::Core).unwrap_or(&false);
    //     let wallet_phase_status = *phases_statuses.get(&SetupPhase::Wallet).unwrap_or(&false);
    //     let hardware_phase_status = *phases_statuses.get(&SetupPhase::Hardware).unwrap_or(&false);
    //     let local_node_phase_status = *phases_statuses
    //         .get(&SetupPhase::LocalNode)
    //         .unwrap_or(&false);
    //     let remote_node_phase_status = *phases_statuses
    //         .get(&SetupPhase::RemoteNode)
    //         .unwrap_or(&false);
    //     let unknown_phase_status = *phases_statuses.get(&SetupPhase::Unknown).unwrap_or(&false);

    //     // todo find better way for reapeted calls

    //     if core_phase_status {
    //         info!(target: LOG_TARGET, "Unlocking app");
    //         self.unlock_app();
    //     };

    //     if core_phase_status
    //         && wallet_phase_status
    //         && (local_node_phase_status || remote_node_phase_status)
    //     {
    //         info!(target: LOG_TARGET, "Unlocking wallet");
    //         self.unlock_wallet();
    //     };

    //     if core_phase_status
    //         && wallet_phase_status
    //         && hardware_phase_status
    //         && (local_node_phase_status || remote_node_phase_status)
    //     {
    //         info!(target: LOG_TARGET, "Unlocking mining");
    //         self.unlock_mining();
    //     };

    //     Ok(())
    // }
}
