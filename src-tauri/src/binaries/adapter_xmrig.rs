use std::path::PathBuf;

use anyhow::Error;
use async_trait::async_trait;
use regex::Regex;
use tauri::api::path::cache_dir;

use crate::{github, progress_tracker::ProgressTracker};

use super::binaries_resolver::{LatestVersionApiAdapter, VersionAsset, VersionDownloadInfo};

pub struct XmrigVersionApiAdapter {}

#[async_trait]
impl LatestVersionApiAdapter for XmrigVersionApiAdapter {
    async fn fetch_releases_list(&self) -> Result<Vec<VersionDownloadInfo>, Error> {
        let releases = github::list_releases("xmrig", "xmrig").await?;
        Ok(releases.clone())
    }

    async fn download_and_get_checksum_path(
        &self,
        directory: &PathBuf,
        download_info: VersionDownloadInfo,
        _: ProgressTracker,
    ) -> Result<PathBuf, Error> {
        // When xmrig is downloaded checksum will be already in its folder so there is no need to download it
        // directory parameter should point to folder where xmrig is extracted
        // file with checksum should be in the same folder as xmrig
        // file name is SHA256SUMS
        // let platform = self.find_version_for_platform(version)?;
        let checksum_path = directory
            .join(format!("xmrig-{}", download_info.version.to_string()))
            .join("SHA256SUMS");

        // // extract and parse checksum from SHA256SUMS file to .sha256

        // let checksums = std::fs::read_to_string(&checksum_path)?;
        // let checksums: Vec<&str> = checksums.split('\n').collect();
        // let checksum = checksums
        //     .iter()
        //     .find(|&c| c.contains(&platform.name))
        //     .ok_or(anyhow::anyhow!("Failed to get checksum"))?;
        // let checksum = checksum.split(' ').collect::<Vec<&str>>()[0];

        // let checksum_path = directory.join(format!("xmrig.sha256", checksum));

        // std::fs::write(&checksum_path, checksum)?;

        Ok(checksum_path)
    }

    fn get_binary_folder(&self) -> PathBuf {
        let binary_folder_path = cache_dir().unwrap().join("com.tari.universe").join("xmrig");

        if !binary_folder_path.exists() {
            std::fs::create_dir_all(&binary_folder_path);
        }

        binary_folder_path
    }

    fn find_version_for_platform(
        &self,
        _version: &VersionDownloadInfo,
    ) -> Result<VersionAsset, anyhow::Error> {
        let mut name_suffix = "";
        if cfg!(target_os = "windows") {
            name_suffix = r".*msvc-win64\.zip";
        }
        if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
            name_suffix = r".*macos-x64\.tar.gz";
        }
        if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            // the x64 seems to work better on the M1
            name_suffix = r".*macos-arm64\.tar.gz";
        }
        if cfg!(target_os = "linux") {
            name_suffix = r".*linux-static-x64\.tar.gz";
        }
        if cfg!(target_os = "freebsd") {
            name_suffix = r".*freebsd-static-x64\.tar.gz";
        }
        if name_suffix.is_empty() {
            panic!("Unsupported OS");
        }

        let reg = Regex::new(name_suffix).unwrap();
        let platform = _version
            .assets
            .iter()
            .find(|a| reg.is_match(&a.name))
            .ok_or(anyhow::anyhow!("Failed to get platform asset"))?;
        Ok(platform.clone())
    }
}
