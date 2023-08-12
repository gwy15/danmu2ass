use std::collections::HashSet;

use actix_web::{web, HttpResponse};
use anyhow::{bail, Context};
use biliapi::Request;
use danmu2ass::{bilibili::DanmakuElem, CanvasConfig, InputType};
use log::info;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "snake_case")]
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

async fn convert(request: web::Json<ConvertRequest>) -> HttpResponse {
    let req = request.into_inner();
    let mut output = Vec::<u8>::new();
    let title = match req.source {
        Source::Xml { content, title } => {
            info!("parsing {} bytes in xml", content.len());
            let parser = danmu2ass::Parser::new(content.as_bytes());
            let r = danmu2ass::convert(
                parser,
                title.clone(),
                &mut output,
                req.config,
                &req.denylist,
            );
            if let Err(e) = r {
                return HttpResponse::BadRequest().json(json!({
                    "errmsg": format!("{e:#?}")
                }));
            }
            let ass_title = title
                .as_str()
                .strip_suffix(".xml")
                .map(ToString::to_string)
                .unwrap_or(title);
            ass_title
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
            log::info!("danmu downloaded, title={}", title);
            let r =
                danmu2ass::convert(danmu, title.clone(), &mut output, req.config, &req.denylist);
            if let Err(e) = r {
                return HttpResponse::BadRequest().json(json!({
                    "errmsg": format!("{e:#?}")
                }));
            }
            title
        }
    };
    let title =
        percent_encoding::percent_encode(title.as_bytes(), percent_encoding::NON_ALPHANUMERIC);
    let content_disposition = format!("attachment; filename=\"{title}.ass\"");
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain; charset=utf-8"))
        .append_header(("Content-Disposition", content_disposition))
        .body(output)
}

type Iter = Box<dyn Iterator<Item = anyhow::Result<danmu2ass::Danmu>>>;

async fn run_input_type(input_type: InputType) -> anyhow::Result<(String, Iter)> {
    let client = biliapi::connection::new_client()?;
    match input_type {
        InputType::File(path) => {
            let file = std::fs::File::open(&path)?;
            let parser = danmu2ass::Parser::new(std::io::BufReader::with_capacity(1 << 20, file));
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("danmu")
                .to_string();
            Ok((filename, Box::new(parser)))
        }
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
            let danmu = danmu.into_iter().map(|i| Ok(i.into()));
            Ok((info.title, Box::new(danmu)))
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
            let danmu = danmu.into_iter().map(|i| Ok(i.into()));
            Ok((title, Box::new(danmu)))
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
            let danmu = danmu.into_iter().map(|i| Ok(i.into()));
            Ok((title, Box::new(danmu)))
        }
        _ => {
            bail!("Unsupported input type");
        }
    }
}

fn files_service() -> actix_files::Files {
    actix_files::Files::new("/", "./static")
        .index_file("index.html")
        .prefer_utf8(true)
}

pub async fn run_server() -> anyhow::Result<()> {
    let port = if portpicker::is_free(8081) {
        8081
    } else {
        portpicker::pick_unused_port().context("pick port failed")?
    };
    let fut = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            // .wrap(actix_cors::Cors::permissive())
            .route("/convert", web::post().to(convert))
            .default_service(files_service())
    })
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
