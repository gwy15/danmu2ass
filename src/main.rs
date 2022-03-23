use anyhow::Result;
use clap::Parser;
use danmu2ass::{CanvasConfig, Cli};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::try_init_timed()?;

    let mut cli = Cli::parse();
    cli.check()?;
    let denylist = cli.denylist()?;
    let canvas_config = cli.canvas_config();

    if cli.xml_file_or_path.is_dir() {
        let path = cli.xml_file_or_path.canonicalize()?;
        log::info!("递归处理目录 {}", path.display());
        let glob = format!("{}/**/*.xml", path.display());

        let targets: Vec<PathBuf> = glob::glob(&glob)?.collect::<Result<_, _>>()?;
        log::info!("共找到 {} 个文件", targets.len());
        if targets.is_empty() {
            anyhow::bail!("没有找到任何文件");
        }

        let (file_count, danmu_count) = targets
            .into_par_iter()
            .map(
                |path| match convert(&path, None, canvas_config.clone(), cli.force, &denylist) {
                    Ok(danmu_count) => (1usize, danmu_count),
                    Err(e) => {
                        log::error!("文件 {} 转换错误：{:?}", path.display(), e);
                        (0, 0)
                    }
                },
            )
            .reduce_with(|a, b| (a.0 + b.0, a.1 + b.1))
            .unwrap();

        log::info!(
            "共转换 {} 个文件，共转换 {} 条弹幕",
            file_count,
            danmu_count
        );
    } else {
        convert(
            &cli.xml_file_or_path,
            cli.ass_file,
            canvas_config,
            cli.force,
            &denylist,
        )?;
    }

    Ok(())
}

fn convert(
    file: &Path,
    output: Option<PathBuf>,
    canvas_config: CanvasConfig,
    force: bool,
    denylist: &Option<HashSet<String>>,
) -> Result<usize> {
    let mut parser = danmu2ass::Parser::from_path(file)?;

    let output = match output {
        Some(output) => output,
        None => {
            let mut path = file.to_path_buf();
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
            log::info!("ASS 文件比 XML 文件新，跳过转换（{}）", file.display());
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
        if let Some(denylist) = denylist.as_ref() {
            if denylist.iter().any(|s| danmu.content.contains(s)) {
                continue;
            }
        }
        if let Some(drawable) = canvas.draw(danmu)? {
            count += 1;
            writer.write(drawable)?;
        }
    }
    log::info!(
        "弹幕数量：{}, 耗时 {:?}（{}）",
        count,
        t.elapsed(),
        file.display()
    );
    Ok(count)
}
