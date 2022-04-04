use super::DanmakuElem;
use anyhow::{Context, Result};
use prost::Message;

const URL: &str = "http://api.bilibili.com/x/v2/dm/web/seg.so";

pub async fn get_danmu_for_cid(cid: i64, segment: i64) -> Result<Vec<DanmakuElem>> {
    let client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.60 Safari/537.36")
        .build()?;

    let resp = client
        .get(URL)
        .query(&[("oid", cid), ("segment_index", segment), ("type", 1)])
        .send()
        .await?;
    // code 304
    match resp.status() {
        reqwest::StatusCode::NOT_MODIFIED => {
            debug!(
                "the request cid={} segment={} returned status code {}",
                cid,
                segment,
                resp.status()
            );
            return Ok(vec![]);
        }
        _ => {}
    }

    let is_json_resp = resp
        .headers()
        .get("content-type")
        .map(|v| v.as_bytes().starts_with(b"application/json"))
        .unwrap_or(false);
    if is_json_resp {
        let _: () = biliapi::requests::BiliResponse::from_response(resp).await?;
        anyhow::bail!("The response should fail");
    } else {
        // parse as pb
        let content = resp.bytes().await?;
        let reply = super::DmSegMobileReply::decode(content).context("请求 body 无法解析为 PB")?;
        Ok(reply.elems)
    }
}
