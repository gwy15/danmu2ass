#![allow(unused_imports)]
use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use danmu2ass::Args;
use std::path::PathBuf;

#[cfg(feature = "web")]
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::try_init_timed()?;

    inner_main().await
}

#[cfg(feature = "web")]
async fn inner_main() -> Result<()> {
    web::run_server().await
}

#[cfg(not(feature = "web"))]
async fn inner_main() -> Result<()> {
    let args = load_args()?;
    let pause = args.pause;

    let ret = args.process().await;
    if pause {
        if let Err(e) = ret.as_ref() {
            println!();
            eprintln!("发生错误：{:?}", e);
        }

        println!("按任意键继续");
        std::io::stdin().read_line(&mut String::new())?;
    }
    ret
}

#[cfg(not(feature = "web"))]
fn load_args() -> Result<Args> {
    let path: PathBuf = "./配置文件.toml".parse()?;

    let mut args = if path.exists() {
        log::info!("加载配置文件 {}，不读取命令行参数", path.display());
        let config = std::fs::read_to_string(&path)
            .with_context(|| format!("读取配置文件 {} 失败", path.display()))?;
        toml::from_str(&config)?
    } else {
        Args::parse()
    };

    args.check()?;

    Ok(args)
}
