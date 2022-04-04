use anyhow::{Context, Result};
use clap::Parser;
use danmu2ass::Args;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::try_init_timed()?;

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
