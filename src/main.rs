use anyhow::Result;
use clap::Parser;
use danmu2ass::{CanvasConfig, Cli};
use std::{fs::File, path::PathBuf};

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::try_init_timed()?;

    let mut cli = Cli::parse();
    cli.check()?;
    let canvas_config = cli.canvas_config();

    if cli.xml_file_or_path.is_dir() {
        let path = cli.xml_file_or_path.canonicalize()?;
        log::info!("递归处理目录 {}", path.display());
        let glob = format!("{}/**/*.xml", path.display());
        let mut file_count = 0;
        let mut danmu_count = 0;
        for entry in glob::glob(&glob)? {
            danmu_count += convert(entry?, None, canvas_config.clone(), cli.force)?;
            file_count += 1;
        }
        log::info!(
            "共转换 {} 个文件，共转换 {} 条弹幕",
            file_count,
            danmu_count
        );
    } else {
        convert(cli.xml_file_or_path, cli.ass_file, canvas_config, cli.force)?;
    }

    Ok(())
}

fn convert(
    file: PathBuf,
    output: Option<PathBuf>,
    canvas_config: CanvasConfig,
    force: bool,
) -> Result<usize> {
    let mut parser = danmu2ass::Parser::from_path(&file)?;

    let output = match output {
        Some(output) => output,
        None => {
            let mut path = file.clone();
            path.set_extension("ass");
            if path.is_dir() {
                anyhow::bail!("输出文件 {} 不能是目录", path.display());
            }
            path
        }
    };
    log::info!("转换 {} => {}", file.display(), output.display());

    // 判断是否需要转换
    if !force {
        let xml_modified = file.metadata()?.modified()?;
        let ass_modified = output.metadata()?.modified()?;
        if xml_modified < ass_modified {
            log::info!("ASS 文件比 XML 文件新，跳过转换");
            return Ok(0);
        }
    }

    let writer = File::create(&output)?;
    let mut writer = danmu2ass::AssWriter::new(writer, canvas_config.clone())?;

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
    log::info!("弹幕数量：{}, 耗时 {:?}", count, t.elapsed());
    Ok(count)
}
