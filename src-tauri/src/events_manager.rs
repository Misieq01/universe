// Copyright 2024. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::HashMap;

use log::{error, info};
use tari_core::transactions::tari_amount::MicroMinotari;
use tauri::{AppHandle, Manager};
use tokio::sync::watch::Receiver;

#[cfg(target_os = "macos")]
use crate::events::CriticalProblemPayload;
#[cfg(target_os = "windows")]
use crate::external_dependencies::RequiredExternalDependency;

use crate::{
    commands::CpuMinerStatus,
    events::{EventType, ResumingAllProcessesPayload, SetupStatusPayload, ShowReleaseNotesPayload},
    events_emitter::EventsEmitter,
    events_service::EventsService,
    gpu_status_file::GpuDevice,
    hardware::hardware_status_monitor::GpuDeviceProperties,
    tasks_tracker::TasksTracker,
    wallet_adapter::WalletState,
    BaseNodeStatus, GpuMinerStatus, UniverseAppState,
};

const LOG_TARGET: &str = "tari::universe::events_manager";

pub struct EventsManager {
    events_service: EventsService,
}

impl EventsManager {
    pub fn new(wallet_state_watch_rx: Receiver<Option<WalletState>>) -> Self {
        Self {
            events_service: EventsService::new(wallet_state_watch_rx),
        }
    }

    pub async fn handle_internal_wallet_loaded_or_created(&self, app: &AppHandle) {
        let wallet_address = app
            .state::<UniverseAppState>()
            .tari_address
            .read()
            .await
            .clone();
        EventsEmitter::emit_wallet_address_update(app, wallet_address).await;
    }

    pub async fn wait_for_initial_wallet_scan(&self, app: &AppHandle, block_height: u64) {
        let events_service = self.events_service.clone();
        let app = app.clone();
        TasksTracker::current().spawn(async move {
            match events_service
                .wait_for_wallet_scan(block_height, 1200)
                .await
            {
                Ok(scanned_wallet_state) => match scanned_wallet_state.balance {
                    Some(balance) => EventsEmitter::emit_wallet_balance_update(&app, balance).await,
                    None => {
                        error!(target: LOG_TARGET, "Wallet Balance is None after initial scanning");
                    }
                },
                Err(e) => {
                    error!(target: LOG_TARGET, "Error waiting for wallet scan: {:?}", e);
                }
            };
        });
    }

    pub async fn handle_new_block_height(&self, app: &AppHandle, block_height: u64) {
        let app_clone = app.clone();
        let events_service = self.events_service.clone();
        TasksTracker::current().spawn(async move {
            match events_service.wait_for_wallet_scan(block_height, 20).await {
                Ok(scanned_wallet_state) => match scanned_wallet_state.balance {
                    Some(balance) => {
                        let coinbase_tx =
                            if balance.pending_incoming_balance.gt(&MicroMinotari::zero()) {
                                events_service
                                    .get_coinbase_transaction_for_last_mined_block(
                                        &app_clone.state::<UniverseAppState>().wallet_manager,
                                        block_height,
                                    )
                                    .await
                            } else {
                                None
                            };
                        EventsEmitter::emit_new_block_mined(
                            &app_clone,
                            block_height,
                            coinbase_tx,
                            balance,
                        )
                        .await;
                    }
                    None => {
                        error!(target: LOG_TARGET, "Wallet balance is None after new block height #{}", block_height);
                    }
                },
                Err(e) => {
                    error!(target: LOG_TARGET, "Error waiting for wallet scan: {:?}", e);
                }
            };
        });
    }

    pub async fn handle_base_node_update(&self, app: &AppHandle, status: BaseNodeStatus) {
        EventsEmitter::emit_base_node_update(app, status).await;
    }

    pub async fn handle_connected_peers_update(
        &self,
        app: &AppHandle,
        connected_peers: Vec<String>,
    ) {
        EventsEmitter::emit_connected_peers_update(app, connected_peers).await;
    }

    #[allow(dead_code)]
    pub async fn handle_gpu_devices_update(
        &self,
        app: &AppHandle,
        gpu_devices: Vec<GpuDeviceProperties>,
    ) {
        let gpu_public_devices = gpu_devices
            .iter()
            .map(|gpu_device| gpu_device.public_properties.clone())
            .collect();

        EventsEmitter::emit_gpu_devices_update(app, gpu_public_devices).await;
    }

    pub async fn handle_cpu_mining_update(&self, app: &AppHandle, status: CpuMinerStatus) {
        EventsEmitter::emit_cpu_mining_update(app, status).await;
    }

    pub async fn handle_gpu_mining_update(&self, app: &AppHandle, status: GpuMinerStatus) {
        EventsEmitter::emit_gpu_mining_update(app, status).await;
    }

    pub async fn handle_app_config_loaded(&self, app: &AppHandle) {
        info!(target: LOG_TARGET, "Loading app config");
        let app_state: tauri::State<'_, UniverseAppState> = app.state::<UniverseAppState>();
        info!(target: LOG_TARGET, "Proposing system language");
        let _unused = app_state
            .config
            .write()
            .await
            .propose_system_language()
            .await;
        info!(target: LOG_TARGET, "Reading app config");
        let app_config = app_state.config.read().await.clone();
        info!(target: LOG_TARGET, "Emitting app config loaded event");
        EventsEmitter::emit_app_config_loaded(app, app_config).await;
    }

    pub async fn handle_close_splash_screen(&self, app: &AppHandle) {
        EventsEmitter::emit_close_splashscreen(app).await;
    }

    pub async fn handle_detected_devices(&self, app: &AppHandle, devices: Vec<GpuDevice>) {
        EventsEmitter::emit_detected_devices(app, devices).await;
    }

    pub async fn handle_detected_available_gpu_engines(
        &self,
        app: &AppHandle,
        engines: Vec<String>,
        selected_engine: String,
    ) {
        EventsEmitter::emit_detected_available_gpu_engines(app, engines, selected_engine).await;
    }

    pub async fn handle_resuming_all_processes(
        &self,
        app: &AppHandle,
        payload: ResumingAllProcessesPayload,
    ) {
        EventsEmitter::emit_resuming_all_processes(app, payload).await;
    }

    pub async fn handle_setup_status(&self, app: &AppHandle, payload: SetupStatusPayload) {
        EventsEmitter::emit_setup_status(app, payload).await;
    }

    pub async fn handle_network_status_update(
        &self,
        app: &AppHandle,
        download_speed: f64,
        upload_speed: f64,
        latency: f64,
        is_too_low: bool,
    ) {
        EventsEmitter::emit_network_status(app, download_speed, upload_speed, latency, is_too_low)
            .await;
    }

    #[cfg(target_os = "macos")]
    pub async fn handle_critical_problem(
        &self,
        app: &AppHandle,
        title: Option<String>,
        description: Option<String>,
    ) {
        EventsEmitter::emit_critical_problem(app, CriticalProblemPayload { title, description })
            .await;
    }

    #[cfg(target_os = "windows")]
    pub async fn handle_missing_application_files(
        &self,
        app: &AppHandle,
        external_dependecies: RequiredExternalDependency,
    ) {
        EventsEmitter::emit_missing_applications(app, external_dependecies).await;
    }

    pub async fn handle_show_release_notes(
        &self,
        app: &AppHandle,
        payload: ShowReleaseNotesPayload,
    ) {
        EventsEmitter::emit_show_release_notes(app, payload).await;
    }

    pub async fn handle_stuck_on_orphan_chain(&self, app: &AppHandle, is_stuck: bool) {
        EventsEmitter::emit_stuck_on_orphan_chain(app, is_stuck).await;
    }
    pub async fn handle_progress_tracker_update(
        &self,
        app: &AppHandle,
        event_type: EventType,
        phase_title: String,
        title: String,
        progress: f64,
        title_params: Option<HashMap<String, String>>,
    ) {
        EventsEmitter::emit_progress_tracker_update(
            app,
            event_type,
            phase_title,
            title,
            progress,
            title_params,
        )
        .await;
    }

    pub async fn handle_core_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_core_phase_finished(app, status).await;
    }

    pub async fn handle_wallet_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_wallet_phase_finished(app, status).await;
    }

    pub async fn handle_hardware_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_hardware_phase_finished(app, status).await;
    }

    pub async fn handle_remote_node_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_remote_node_phase_finished(app, status).await;
    }
    pub async fn handle_local_node_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_local_node_phase_finished(app, status).await;
    }
    pub async fn handle_unknown_phase_finished(&self, app: &AppHandle, status: bool) {
        EventsEmitter::emit_unknown_phase_finished(app, status).await;
    }
    pub async fn handle_unlock_app(&self, app: &AppHandle) {
        EventsEmitter::emit_unlock_app(app).await;
    }

    pub async fn handle_unlock_wallet(&self, app: &AppHandle) {
        EventsEmitter::emit_unlock_wallet(app).await;
    }

    pub async fn handle_unlock_mining(&self, app: &AppHandle) {
        EventsEmitter::emit_unlock_mining(app).await;
    }
}
