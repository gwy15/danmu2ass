use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(help = "需要转换的 XML 文件")]
    pub xml_file: PathBuf,

    #[clap(help = "输出的 ASS 文件，默认为输入文件名将 .xml 替换为 .ass")]
    pub ass_file: Option<PathBuf>,
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
}
