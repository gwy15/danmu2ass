use anyhow::{Context, Result};
use serde::Deserialize;

/// 这里放在了 result 里面
#[derive(Debug, Deserialize)]
pub struct BiliResponse<T> {
    code: i64,

    #[serde(default)]
    message: String,

    #[serde(default = "Option::default")]
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct Season {
    pub season_id: u64,
    pub season_title: String,
    pub media_id: u64,

    pub title: String,
    pub cover: String,

    pub episodes: Vec<Episode>,
}

#[derive(Debug, Deserialize)]
pub struct Episode {
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    /// epid
    pub id: u64,

    #[serde(rename = "duration")]
    pub duration_ms: u64,

    pub title: String,
}

impl Season {
    pub async fn request(client: &reqwest::Client, args: (&'static str, u64)) -> Result<Self> {
        let request = client
            .get("https://api.bilibili.com/pgc/view/web/season")
            .query(&[args])
            .send();
        let response = request.await?;

        if !response.status().is_success() {
            let status = response.status();
            anyhow::bail!(
                "status = {:?}, response text = {:?}",
                status,
                response.text().await
            );
        }
        let response_text = response.text().await?;
        let this: BiliResponse<Self> =
            serde_json::from_str(&response_text).context("serde error")?;
        if this.code != 0 {
            debug!("response text = {}", response_text);
            anyhow::bail!("code = {}, message = {}", this.code, this.message);
        }
        this.result.context("result 为空")
    }
}
