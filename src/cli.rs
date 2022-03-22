use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author = "gwy15", version, about = "将 XML 弹幕转换为 ASS 文件")]
pub struct Cli {
    #[clap(help = "需要转换的 XML 文件")]
    pub xml_file: PathBuf,

    #[clap(
        long = "output",
        short = 'o',
        help = "输出的 ASS 文件，默认为输入文件名将 .xml 替换为 .ass"
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
        default_value = "sans-serif"
    )]
    font: String,

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
}

impl Cli {
    pub fn check(&mut self) -> Result<()> {
        if self.xml_file.is_dir() {
            bail!("{} 是一个目录", self.xml_file.display());
        }

        if self.ass_file.is_none() {
            let mut path = self.xml_file.clone();
            path.set_extension("ass");

            if path.is_dir() {
                bail!("{} 是一个目录", path.display());
            }

            self.ass_file = Some(path);
        }

        Ok(())
    }

    pub fn canvas_config(&self) -> crate::CanvasConfig {
        let config = crate::CanvasConfig {
            width: self.width,
            height: self.height,
            font: self.font.clone(),
            duration: self.duration,
            lane_size: self.lane_size,
            float_percentage: self.float_percentage,
            opacity: ((1.0 - self.alpha) * 255.0) as u8,
            bottom_percentage: 0.3,
        };

        config
    }
}
