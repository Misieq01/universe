use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};
use std::{
    sync::{LazyLock, Mutex},
    time::SystemTime,
};

use crate::{app_config::AirdropTokens, AppConfig};

use super::trait_config::{ConfigContentImpl, ConfigImpl};

static INSTANCE: LazyLock<Mutex<ConfigCore>> = LazyLock::new(|| Mutex::new(ConfigCore::new()));

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
#[derive(Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct ConfigCoreContent {
    created_at: SystemTime,
    is_p2pool_enabled: bool,
    use_tor: bool,
    allow_telemetry: bool,
    last_binaries_update_timestamp: Option<SystemTime>,
    anon_id: Option<String>,
    should_auto_launch: bool,
    mmproxy_use_monero_failover: bool,
    mmproxy_monero_nodes: Vec<String>,
    auto_update: bool,
    p2pool_stats_server_port: Option<u16>,
    pre_release: bool,
    last_changelog_version: Option<String>,
    airdrop_tokens: Option<AirdropTokens>,
}

impl Default for ConfigCoreContent {
    fn default() -> Self {
        Self {
            created_at: SystemTime::now(),
            is_p2pool_enabled: false,
            use_tor: false,
            allow_telemetry: true,
            last_binaries_update_timestamp: None,
            anon_id: None,
            should_auto_launch: false,
            mmproxy_use_monero_failover: false,
            mmproxy_monero_nodes: vec![],
            auto_update: true,
            p2pool_stats_server_port: None,
            pre_release: false,
            last_changelog_version: None,
            airdrop_tokens: None,
        }
    }
}
impl ConfigContentImpl for ConfigCoreContent {}

pub struct ConfigCore {
    content: ConfigCoreContent,
}

impl ConfigImpl for ConfigCore {
    type Config = ConfigCoreContent;
    type OldConfig = AppConfig;

    fn current() -> &'static Mutex<Self> {
        &INSTANCE
    }

    fn new() -> Self {
        Self {
            content: ConfigCoreContent::default(),
        }
    }

    fn get_name() -> String {
        "core_config".to_string()
    }

    fn get_content(&self) -> &Self::Config {
        &self.content
    }

    fn get_content_mut(&mut self) -> &mut Self::Config {
        &mut self.content
    }

    fn migrate_old_config(&mut self, old_config: Self::OldConfig) -> Result<(), anyhow::Error> {
        self.content = ConfigCoreContent {
            created_at: SystemTime::now(),
            is_p2pool_enabled: old_config.p2pool_enabled(),
            use_tor: old_config.use_tor(),
            allow_telemetry: old_config.allow_telemetry(),
            last_binaries_update_timestamp: Some(old_config.last_binaries_update_timestamp()),
            anon_id: Some(old_config.anon_id().to_string()),
            should_auto_launch: old_config.should_auto_launch(),
            mmproxy_use_monero_failover: old_config.mmproxy_use_monero_fail(),
            mmproxy_monero_nodes: old_config.mmproxy_monero_nodes().to_vec(),
            auto_update: old_config.auto_update(),
            p2pool_stats_server_port: old_config.p2pool_stats_server_port(),
            pre_release: old_config.pre_release(),
            last_changelog_version: Some(old_config.last_changelog_version().to_string()),
            airdrop_tokens: old_config.airdrop_tokens(),
        };
        Ok(())
    }
}
