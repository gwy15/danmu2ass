use super::DanmakuElem;
use anyhow::{Context, Result};
use prost::Message;

const URL: &str = "http://api.bilibili.com/x/v2/dm/web/seg.so";

async fn get_danmu_for_cid_segment(
    client: reqwest::Client,
    cid: u64,
    segment: u64,
) -> Result<Vec<DanmakuElem>> {
    let resp = client
        .get(URL)
        .query(&[("oid", cid), ("segment_index", segment), ("type", 1)])
        .send()
        .await?;
    // code 304
    if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
        debug!(
            "the request cid={} segment={} returned status code {}",
            cid,
            segment,
            resp.status()
        );
        return Ok(vec![]);
    }

    let is_json_resp = resp
        .headers()
        .get("content-type")
        .map(|v| v.as_bytes().starts_with(b"application/json"))
        .unwrap_or(false);
    if is_json_resp {
        biliapi::requests::BiliResponse::<()>::from_response(resp).await?;
        anyhow::bail!("The response should fail");
    } else {
        // parse as pb
        let content = resp.bytes().await?;
        let reply = super::DmSegMobileReply::decode(content).context("请求 body 无法解析为 PB")?;
        Ok(reply.elems)
    }
}

fn div_ceil(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

pub async fn get_danmu_for_video(cid: u64, duration_sec: u64) -> Result<Vec<DanmakuElem>> {
    let client = reqwest::ClientBuilder::new()
    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.60 Safari/537.36")
    .build()?;

    // segment 为 6 分钟一包，见 https://github.com/SocialSisterYi/bilibili-API-collect/blob/master/danmaku/danmaku_proto.md
    let s = duration_sec;
    info!("获取视频 aid={} 的弹幕，视频有 {} 秒", cid, s);

    let mut fut = vec![];
    for i in 0..div_ceil(s, 360) {
        fut.push(get_danmu_for_cid_segment(client.clone(), cid, i + 1));
    }
    let mut results =
        futures::future::try_join_all(fut)
            .await?
            .into_iter()
            .fold(vec![], |mut acc, v| {
                acc.extend(v);
                acc
            });
    results.sort_unstable_by_key(|d| d.progress);

    Ok(results)
}
