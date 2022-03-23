use std::{collections::HashSet, path::PathBuf};

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "gwy15", version, about = "将 XML 弹幕转换为 ASS 文件")]
pub struct Cli {
    #[clap(
        help = "需要转换的 XML 文件或文件夹，如果是文件夹会递归将旗下所有 XML 都进行转换",
        default_value = "."
    )]
    pub xml_file_or_path: PathBuf,

    #[clap(
        long = "output",
        short = 'o',
        help = "输出的 ASS 文件，默认为输入文件名将 .xml 替换为 .ass，如果输入是文件夹则忽略"
    )]
    pub ass_file: Option<PathBuf>,

    #[clap(long = "width", short = 'w', help = "屏幕宽度", default_value = "1280")]
    width: u32,

    #[clap(long = "height", short = 'h', help = "屏幕宽度", default_value = "720")]
    height: u32,

    #[clap(
        long = "font",
        short = 'f',
        help = "弹幕使用字体",
        default_value = "黑体"
    )]
    font: String,

    #[clap(long = "font-size", help = "弹幕字体大小", default_value = "25")]
    font_size: u32,

    #[clap(
        long = "font-ratio",
        help = "计算弹幕宽度时的比例，如果你的字体很宽为避免重叠需要调大这个数值",
        default_value = "1.2"
    )]
    width_ratio: f64,

    #[clap(
        long = "duration",
        short = 'd',
        help = "弹幕在屏幕上的持续时间，单位为s，可以有小数",
        default_value = "15"
    )]
    duration: f64,

    #[clap(
        long = "lane-size",
        short = 'l',
        help = "弹幕所占据的高度",
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
}

impl Cli {
    pub fn check(&mut self) -> Result<()> {
        if self.xml_file_or_path.is_dir() {
            info!("输入是目录，将递归遍历目录下所有 XML 文件");
        }
        if let Some(f) = self.denylist.as_ref() {
            if !f.exists() {
                anyhow::bail!("黑名单文件不存在");
            }
            if f.is_dir() {
                anyhow::bail!("黑名单文件不能是目录");
            }
        }

        Ok(())
    }

    pub fn canvas_config(&self) -> crate::CanvasConfig {
        crate::CanvasConfig {
            width: self.width,
            height: self.height,
            font: self.font.clone(),
            font_size: self.font_size,
            width_ratio: self.width_ratio,
            duration: self.duration,
            lane_size: self.lane_size,
            float_percentage: self.float_percentage,
            opacity: ((1.0 - self.alpha) * 255.0) as u8,
            bottom_percentage: 0.3,
        }
    }

    pub fn denylist(&self) -> Result<Option<HashSet<String>>> {
        match self.denylist.as_ref() {
            None => Ok(None),
            Some(path) => {
                let denylist = std::fs::read_to_string(path)?;
                let list = denylist.split('\n').map(|s| s.trim().to_string()).collect();
                Ok(Some(list))
            }
        }
    }
}
