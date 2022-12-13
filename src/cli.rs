use std::{
    collections::HashSet,
    fs::File,
    path::{Path, PathBuf},
};

use super::input_type::InputType;
use super::CanvasConfig;
use anyhow::{Context, Result};
use biliapi::Request;
use clap::Parser;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug, serde::Deserialize)]
#[clap(author = "gwy15", version, about = "将 XML 弹幕转换为 ASS 文件")]
pub struct Args {
    #[clap(
        help = "需要转换的输入，可以是 xml 文件、文件夹或是哔哩哔哩链接、BV 号。如果是文件夹会递归将其下所有 XML 都进行转换",
        default_value = "."
    )]
    pub input: String,

    #[clap(
        long = "output",
        short = 'o',
        help = "输出的 ASS 文件，默认为输入文件名将 .xml 替换为 .ass，如果输入是文件夹则忽略"
    )]
    pub ass_file: Option<PathBuf>,

    #[clap(long = "width", short = 'w', help = "屏幕宽度", default_value = "1280")]
    width: u32,

    #[clap(long = "height", short = 'h', help = "屏幕高度", default_value = "720")]
    height: u32,

    #[clap(
        long = "font",
        short = 'f',
        help = "弹幕使用字体。单位：像素",
        default_value = "黑体"
    )]
    font: String,

    #[clap(long = "font-size", help = "弹幕字体大小", default_value = "25")]
    font_size: u32,

    #[clap(
        long = "width-ratio",
        help = "计算弹幕宽度的比例，为避免重叠可以调大这个数值",
        default_value = "1.2"
    )]
    width_ratio: f64,

    #[clap(
        long = "horizontal-gap",
        help = "每条弹幕之间的最小水平间距，为避免重叠可以调大这个数值。单位：像素",
        default_value = "20.0"
    )]
    #[serde(default)]
    horizontal_gap: f64,

    #[clap(
        long = "duration",
        short = 'd',
        help = "弹幕在屏幕上的持续时间，单位为秒，可以有小数",
        default_value = "15"
    )]
    duration: f64,

    #[clap(
        long = "lane-size",
        short = 'l',
        help = "弹幕所占据的高度，即“行高度/行间距”",
        default_value = "32"
    )]
    lane_size: u32,

    #[clap(
        long = "float-percentage",
        short = 'p',
        help = "屏幕上滚动弹幕最多高度百分比",
        default_value = "0.5"
    )]
    float_percentage: f64,

    #[clap(
        long = "alpha",
        short = 'a',
        help = "弹幕不透明度",
        default_value = "0.7"
    )]
    alpha: f64,

    #[clap(
        long = "force",
        help = "默认会跳过 ass 比 xml 修改时间更晚的文件，此参数会强制转换"
    )]
    pub force: bool,

    #[clap(
        long = "denylist",
        help = "黑名单，需要过滤的关键词列表文件，每行一个关键词"
    )]
    denylist: Option<PathBuf>,

    #[clap(long = "pause", help = "在处理完后暂停等待输入")]
    pub pause: bool,

    #[clap(long = "outline", help = "描边宽度", default_value = "0.8")]
    pub outline: f64,

    #[clap(long = "bold", help = "加粗")]
    #[serde(default)]
    pub bold: bool,

    #[clap(
        long = "time-offset",
        help = "时间轴偏移，>0 会让弹幕延后，<0 会让弹幕提前，单位为秒",
        default_value = "0.0"
    )]
    #[serde(default)]
    pub time_offset: f64,
}

impl Args {
    pub fn check(&mut self) -> Result<()> {
        if let Some(f) = self.denylist.as_ref() {
            if !f.exists() {
                anyhow::bail!("黑名单文件不存在");
            }
            if f.is_dir() {
                anyhow::bail!("黑名单文件不能是目录");
            }
        }
        if self.float_percentage < 0.0 {
            anyhow::bail!("滚动弹幕最大高度百分比不能小于 0");
        }
        if self.float_percentage > 1.0 {
            anyhow::bail!("滚动弹幕最大高度百分比不能大于 1");
        }

        Ok(())
    }

    fn canvas_config(&self) -> crate::CanvasConfig {
        crate::CanvasConfig {
            width: self.width,
            height: self.height,
            font: self.font.clone(),
            font_size: self.font_size,
            width_ratio: self.width_ratio,
            horizontal_gap: self.horizontal_gap,
            duration: self.duration,
            lane_size: self.lane_size,
            float_percentage: self.float_percentage,
            opacity: ((1.0 - self.alpha) * 255.0) as u8,
            bottom_percentage: 0.3,
            outline: self.outline,
            bold: u8::from(self.bold),
            time_offset: self.time_offset,
        }
    }

    fn denylist(&self) -> Result<Option<HashSet<String>>> {
        match self.denylist.as_ref() {
            None => Ok(None),
            Some(path) => {
                let denylist = std::fs::read_to_string(path)?;
                let list: HashSet<String> = denylist
                    .split('\n')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                log::info!("黑名单载入 {} 个", list.len());
                log::debug!("黑名单：{:?}", list);
                Ok(Some(list))
            }
        }
    }

    pub async fn process(self) -> Result<()> {
        match self.input.parse::<InputType>()? {
            InputType::File(file) => {
                let denylist = self.denylist()?;
                let canvas_config = self.canvas_config();
                convert_xml(&file, self.ass_file, self.force, canvas_config, &denylist)?;
            }
            InputType::Folder(path) => {
                self.process_folder(path)?;
            }
            InputType::BV { bv, p } => {
                self.process_bv(bv, p).await?;
            }
            InputType::Season { season_id } => {
                self.process_episode_or_season("season_id", season_id)
                    .await?;
            }
            InputType::Episode { episode_id } => {
                self.process_episode_or_season("ep_id", episode_id).await?;
            }
        }

        Ok(())
    }

    fn process_folder(&self, folder: PathBuf) -> Result<()> {
        let canvas_config = self.canvas_config();
        let denylist = self.denylist()?;

        // Windows 下 canonicalize 会莫名其妙，见 https://stackoverflow.com/questions/1816691/how-do-i-resolve-a-canonical-filename-in-windows
        #[cfg(not(windows))]
        let folder = folder.canonicalize()?;

        log::info!("递归处理目录 {}", folder.display());
        let glob = format!("{}/**/*.xml", folder.display());

        let targets: Vec<PathBuf> = glob::glob(&glob)?.collect::<Result<_, _>>()?;
        log::info!("共找到 {} 个文件", targets.len());
        if targets.is_empty() {
            anyhow::bail!("没有找到任何文件");
        }

        let t = std::time::Instant::now();
        let (file_count, danmu_count) = targets
            .into_par_iter()
            .map(|path| {
                match convert_xml(&path, None, self.force, canvas_config.clone(), &denylist) {
                    Ok(danmu_count) => (1usize, danmu_count),
                    Err(e) => {
                        log::error!("文件 {} 转换错误：{:?}", path.display(), e);
                        (0, 0)
                    }
                }
            })
            .reduce_with(|a, b| (a.0 + b.0, a.1 + b.1))
            .unwrap();

        log::info!(
            "共转换 {} 个文件，共转换 {} 条弹幕，耗时 {:?}",
            file_count,
            danmu_count,
            t.elapsed()
        );
        Ok(())
    }

    async fn process_bv(&self, bv: String, p: Option<u32>) -> Result<()> {
        let p = p.unwrap_or(1);
        // get info for video
        let client = biliapi::connection::new_client()?;
        let mut info = biliapi::requests::VideoInfo::request(&client, bv.clone()).await?;
        if p > info.pages.len() as u32 {
            anyhow::bail!("视频 {} 只有 {} p，指定 {}p", bv, info.pages.len(), p);
        }
        let page = info.pages.swap_remove(p as usize - 1);

        let danmu = crate::bilibili::get_danmu_for_video(page.cid, page.duration.as_secs()).await?;
        let danmu = danmu.into_iter().map(|d| Ok(d.into()));

        let ass = PathBuf::from(format!("{}.ass", info.title));
        convert(danmu, &ass, self.canvas_config(), &self.denylist()?)?;

        Ok(())
    }

    /// key_type: season_id 或是 ep_id
    async fn process_episode_or_season(
        &self,
        key_type: &'static str,
        ep_or_season_id: u64,
    ) -> Result<()> {
        let client = biliapi::connection::new_client()?;

        let mut season_info =
            crate::bilibili::Season::request(&client, (key_type, ep_or_season_id))
                .await
                .context("获取 season 失败")?;
        let (title, ep) = match key_type {
            "season_id" => (season_info.title, season_info.episodes.swap_remove(0)),
            "ep_id" => {
                let ep = season_info
                    .episodes
                    .into_iter()
                    .find(|ep| ep.id == ep_or_season_id)
                    .ok_or_else(|| anyhow::anyhow!("没有找到 ep_id {}", ep_or_season_id))?;
                (format!("{} - {}", season_info.title, ep.title), ep)
            }
            _ => unreachable!(),
        };

        let danmu = crate::bilibili::get_danmu_for_video(ep.cid, ep.duration_ms / 1000).await?;
        let danmu = danmu.into_iter().map(|d| Ok(d.into()));

        let ass = PathBuf::from(format!("{}.ass", title));
        convert(danmu, &ass, self.canvas_config(), &self.denylist()?)?;

        Ok(())
    }
}

fn convert_xml(
    file: &Path,
    output: Option<PathBuf>,
    force: bool,
    canvas_config: CanvasConfig,
    denylist: &Option<HashSet<String>>,
) -> Result<usize> {
    if !file.exists() {
        anyhow::bail!("文件 {} 不存在", file.display());
    }

    let output = output.unwrap_or_else(|| file.with_extension("ass"));
    log::info!("转换 {} => {}", file.display(), output.display());
    // 判断是否需要转换
    if !force && output.exists() {
        let xml_modified = file.metadata()?.modified()?;
        let ass_modified = output.metadata()?.modified()?;
        if xml_modified < ass_modified {
            log::info!("ASS 文件比 XML 文件新，跳过转换（{}）", file.display());
            return Ok(0);
        }
    }

    let data_provider = crate::Parser::from_path(file)?;

    convert(data_provider, &output, canvas_config, denylist)
}

fn convert<I>(
    data_provider: I,
    output: &Path,
    canvas_config: CanvasConfig,
    denylist: &Option<HashSet<String>>,
) -> Result<usize>
where
    I: Iterator<Item = Result<crate::Danmu>>,
{
    if output.is_dir() {
        anyhow::bail!("输出文件 {} 不能是目录", output.display());
    }

    let title = output
        .file_stem()
        .context("无法解析出文件名")?
        .to_string_lossy()
        .to_string();
    let writer = File::create(output).context("创建输出文件错误")?;
    let mut writer = super::AssWriter::new(writer, title, canvas_config.clone())?;

    let mut count = 0;
    let mut canvas = canvas_config.canvas();
    let t = std::time::Instant::now();

    for danmu in data_provider {
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
        output.display()
    );
    Ok(count)
}
