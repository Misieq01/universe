use log::info;
use serde::Deserialize;

const LOG_TARGET: &str = "tari::universe::xmrig::latest_release";
#[derive(Debug, Deserialize)]
pub struct Asset {
    id: String,
    pub(crate) name: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct XmrigRelease {
    pub(crate) version: String,
    assets: Vec<Asset>,
}

impl XmrigRelease {
    pub fn get_asset(&self, id: &str) -> Option<&Asset> {
        for asset in &self.assets {
            info!(target: LOG_TARGET, "Checking asset {:?}", asset);
            if asset.id == id {
                return Some(asset);
            }
        }
        None
    }
}

pub async fn fetch_latest_release() -> Result<XmrigRelease, anyhow::Error> {
    let url = "https://api.xmrig.com/1/latest_release";
    let response = reqwest::get(url).await?;
    let latest_release: XmrigRelease = response.json().await?;
    Ok(latest_release)
}
