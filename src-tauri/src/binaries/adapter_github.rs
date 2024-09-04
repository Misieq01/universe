use std::path::PathBuf;

use anyhow::Error;
use async_trait::async_trait;
use log::info;
use regex::Regex;
use tauri::{api::path::cache_dir, utils::platform};

use crate::github;

use super::binaries_resolver::{
    LatestVersionApiAdapter, VersionAsset, VersionDownloadInfo, BINARY_RESOLVER_LOG_TARGET,
};

pub struct GithubReleasesAdapter {
    pub repo: String,
    pub owner: String,
    pub specific_name: Option<Regex>,
}

#[async_trait]
impl LatestVersionApiAdapter for GithubReleasesAdapter {
    async fn fetch_releases_list(&self) -> Result<Vec<VersionDownloadInfo>, Error> {
        let releases = github::list_releases(&self.owner, &self.repo).await?;
        Ok(releases.clone())
    }

    // async fn get_checksum_path(&self, version: &VersionDownloadInfo) -> Option<PathBuf> {
    //     let platform = self.find_version_for_platform(version);

    //     if platform.is_err() {
    //         info!(target: BINARY_RESOLVER_LOG_TARGET, "Failed to get platform asset");
    //         return None;
    //     }

    //     let version_dir = self
    //         .get_binary_folder()
    //         .join(version.clone().version.to_string())

    //     let checksum_path = version_dir.join(format!("{}.sha256", platform.unwrap().name));

    //     match checksum_path.exists() {
    //         true => Some(checksum_path),
    //         false => None,
    //     }
    // }

    // fn  get_binary_file(&self, version: &VersionDownloadInfo) -> Option<PathBuf> {
    //     let platform = self.find_version_for_platform(version);

    //     if platform.is_err() {
    //         info!(target: BINARY_RESOLVER_LOG_TARGET, "Failed to get platform asset");
    //         return None;
    //     }

    //     let binary_path = self
    //         .get_binary_folder()
    //         .join(version.clone().version.to_string())
    //         .join(format!("{}.zip", platform.unwrap().name));

    //     match binary_path.exists() {
    //         true => Some(binary_path),
    //         false => None,
    //     }
    // }

    fn get_binary_folder(&self) -> PathBuf {
        let binary_folder_path = cache_dir()
            .unwrap()
            .join("com.tari.universe")
            .join(&self.repo);

        if !binary_folder_path.exists() {
            std::fs::create_dir_all(&binary_folder_path);
        }

        binary_folder_path
    }

    fn find_version_for_platform(
        &self,
        version: &VersionDownloadInfo,
    ) -> Result<VersionAsset, Error> {
        let mut name_suffix = "";
        // TODO: add platform specific logic
        if cfg!(target_os = "windows") {
            name_suffix = r"windows-x64.*\.zip";
        }

        if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            name_suffix = r"macos-x86_64.*\.zip";
        }

        if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            name_suffix = r"macos-arm64.*\.zip";
        }
        if cfg!(target_os = "linux") {
            name_suffix = r"linux-x86_64.*\.zip";
        }

        info!(target: BINARY_RESOLVER_LOG_TARGET, "Looking for platform with suffix: {}", name_suffix);

        let reg = Regex::new(name_suffix).unwrap();

        let platform = version
            .assets
            .iter()
            .find(|a| {
                println!("Asset name: {}", a.name);
                if let Some(ref specific) = self.specific_name {
                    specific.is_match(&a.name) && reg.is_match(&a.name)
                } else {
                    reg.is_match(&a.name)
                }
            })
            .ok_or(anyhow::anyhow!("Failed to get platform asset"))?;
        info!(target: BINARY_RESOLVER_LOG_TARGET, "Found platform: {:?}", platform);
        Ok(platform.clone())
    }
}
