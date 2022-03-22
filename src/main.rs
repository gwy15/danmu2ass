use anyhow::{Context, Result};
use clap::Parser;
use danmu2ass::Cli;
use std::fs::File;

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;

    let mut cli = Cli::parse();
    cli.check()?;

    let canvas_config = cli.canvas_config();

    let mut parser = danmu2ass::Parser::from_path(&cli.xml_file)?;
    let writer = File::create(&cli.ass_file.context("ass_file 为空")?)?;
    let mut writer = danmu2ass::AssWriter::new(writer, &canvas_config)?;

    let t = std::time::Instant::now();
    let mut count = 0;
    let mut canvas = canvas_config.canvas();
    #[cfg(feature = "quick_xml")]
    let mut buf = Vec::new();
    while let Some(danmu) = parser.next(
        #[cfg(feature = "quick_xml")]
        &mut buf,
    ) {
        let danmu = danmu?;
        if let Some(drawable) = canvas.draw(danmu)? {
            count += 1;
            writer.write(drawable)?;
        }
    }
    println!("弹幕数量：{}, 耗时 {:?}", count, t.elapsed());

    Ok(())
}
