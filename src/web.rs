use std::collections::HashSet;

use actix_web::{web, HttpResponse};
use anyhow::{bail, Context};
use biliapi::Request;
use danmu2ass::{bilibili::DanmakuElem, CanvasConfig, InputType};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum Source {
    Xml { content: String, title: String },
    Url { url: String },
}

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    source: Source,
    config: CanvasConfig,
    denylist: Option<HashSet<String>>,
}

#[actix_web::get("/convert}")]
async fn convert(request: web::Json<ConvertRequest>) -> HttpResponse {
    let req = request.into_inner();
    match req.source {
        Source::Xml { content, title } => {
            let parser = danmu2ass::Parser::new(content.as_bytes());
            let mut output = Vec::<u8>::new();
            let r = danmu2ass::convert(parser, title, &mut output, req.config, &req.denylist);
            if let Err(e) = r {
                return HttpResponse::BadRequest().json(json!({
                    "errmsg": format!("{e:#?}")
                }));
            }
            HttpResponse::Ok().content_type("text/plain").body(output)
        }
        Source::Url { url } => {
            let input_type: InputType = url.parse().unwrap();
            let r = run_input_type(input_type).await;
            let (title, danmu) = match r {
                Ok((title, danmu)) => (title, danmu),
                Err(e) => {
                    return HttpResponse::BadRequest().json(json!({
                        "errmsg": format!("{e:#?}")
                    }));
                }
            };
            let mut output = Vec::<u8>::new();
            let r = danmu2ass::convert(
                danmu.into_iter().map(|i| Ok(i.into())),
                title,
                &mut output,
                req.config,
                &req.denylist,
            );
            if let Err(e) = r {
                return HttpResponse::BadRequest().json(json!({
                    "errmsg": format!("{e:#?}")
                }));
            }
            HttpResponse::Ok().content_type("text/plain").body(output)
        }
    }
}

async fn run_input_type(input_type: InputType) -> anyhow::Result<(String, Vec<DanmakuElem>)> {
    let client = biliapi::connection::new_client()?;
    match input_type {
        InputType::BV { bv, p } => {
            let p = p.unwrap_or(1);
            // get info for video
            let mut info = biliapi::requests::VideoInfo::request(&client, bv.clone()).await?;
            if p > info.pages.len() as u32 {
                anyhow::bail!("视频 {} 只有 {} p，指定 {}p", bv, info.pages.len(), p);
            }
            let page = info.pages.swap_remove(p as usize - 1);

            let danmu =
                danmu2ass::bilibili::get_danmu_for_video(page.cid, page.duration.as_secs()).await?;
            Ok((info.title, danmu))
        }
        InputType::Season { season_id } => {
            let mut season_info =
                danmu2ass::bilibili::Season::request(&client, ("season_id", season_id))
                    .await
                    .context("获取 season 失败")?;
            let title = season_info.title;
            let episode = season_info.episodes.swap_remove(0);
            let danmu =
                danmu2ass::bilibili::get_danmu_for_video(episode.cid, episode.duration_ms / 1000)
                    .await?;
            Ok((title, danmu))
        }
        InputType::Episode { episode_id } => {
            let season_info = danmu2ass::bilibili::Season::request(&client, ("ep_id", episode_id))
                .await
                .context("获取 season 失败")?;
            let ep = season_info
                .episodes
                .into_iter()
                .find(|ep| ep.id == episode_id)
                .ok_or_else(|| anyhow::anyhow!("没有找到 ep_id {}", episode_id))?;
            let title = format!("{} - {}", season_info.title, ep.title);
            let danmu =
                danmu2ass::bilibili::get_danmu_for_video(ep.cid, ep.duration_ms / 1000).await?;
            Ok((title, danmu))
        }
        _ => {
            bail!("Unsupported input type");
        }
    }
}

pub async fn run_server() -> anyhow::Result<()> {
    let port = portpicker::pick_unused_port().context("pick port failed")?;
    let fut = actix_web::HttpServer::new(|| actix_web::App::new().service(convert))
        .bind(("127.0.0.1", port))?
        .run();
    let handle = tokio::spawn(fut);
    // open
    match open::that(format!("http://127.0.0.1:{port}")) {
        Ok(_) => {
            handle.await??;
        }
        Err(e) => {
            handle.abort();
            return Err(anyhow::anyhow!("open browser failed: {}", e));
        }
    }
    Ok(())
}
