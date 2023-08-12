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

    let args = load_args()?;
    #[cfg(feature = "web")]
    if !args.no_web {
        return web::run_server().await;
    }

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

fn load_args() -> Result<Args> {
    let mut args = Args::parse();
    args.check()?;
    Ok(args)
}
