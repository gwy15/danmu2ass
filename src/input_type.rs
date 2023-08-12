use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum InputType {
    File(PathBuf),
    Folder(PathBuf),
    /// 如 `https://www.bilibili.com/video/BV1z44y1E7m6`
    BV {
        bv: String,
        p: Option<u32>,
    },
    /// 如 `https://www.bilibili.com/bangumi/play/ss28296`
    Season {
        season_id: u64,
    },
    /// 如 `https://www.bilibili.com/bangumi/play/ep473502`
    Episode {
        episode_id: u64,
    },
}

impl std::str::FromStr for InputType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http") {
            if let Ok(url) = url::Url::parse(s) {
                info!("输入类型为 URL，解析中...");
                return Self::from_url(url);
            }
        }
        if s.chars().all(|c| c.is_ascii_alphanumeric()) {
            if s.starts_with("BV") {
                return Ok(InputType::BV {
                    bv: s.to_string(),
                    p: None,
                });
            }
            if let Ok(t) = Self::from_episode_or_season_str(s) {
                return Ok(t);
            }
        }

        let path = PathBuf::from(s);
        if path.is_dir() {
            Ok(InputType::Folder(path))
        } else {
            Ok(InputType::File(path))
        }
    }
}

impl InputType {
    pub fn from_url(url: url::Url) -> Result<Self> {
        if url.domain() != Some("www.bilibili.com") {
            anyhow::bail!("不支持的域名 {}", url.domain().unwrap_or(""));
        }
        let mut path = url
            .path_segments()
            .context("解析 URL 的 path segments 错误")?;
        let first_segment = path.next().context("解析 URL 的 path segments 错误")?;
        match first_segment {
            "video" => {
                let bv = path
                    .next()
                    .context("解析 URL 的 path segments 错误")?
                    .to_string();
                let p = url
                    .query_pairs()
                    .find(|(k, _)| k == "p")
                    .and_then(|(_, v)| v.parse().ok());
                Ok(InputType::BV { bv, p })
            }
            "bangumi" => {
                anyhow::ensure!(
                    path.next() == Some("play"),
                    "不合法的 URL，应该是 bangumi/play"
                );
                let id = path.next().context("解析 URL 的 path segments 错误")?;
                InputType::from_episode_or_season_str(id)
            }
            _ => {
                anyhow::bail!("不支持的 URL，应该是 video/BV1z44y1E7m6 或 bangumi/play/ss28296 或 bangumi/play/ep473502");
            }
        }
    }

    pub fn from_episode_or_season_str(s: &str) -> Result<Self> {
        match s.chars().take(2).collect::<String>().as_str() {
            "ss" => {
                let season_id = s
                    .chars()
                    .skip(2)
                    .collect::<String>()
                    .parse()
                    .with_context(|| format!("解析 id {} 错误", s))?;
                Ok(InputType::Season { season_id })
            }
            "ep" => {
                let episode_id = s
                    .chars()
                    .skip(2)
                    .collect::<String>()
                    .parse()
                    .with_context(|| format!("解析 id {} 错误", s))?;
                Ok(InputType::Episode { episode_id })
            }
            _ => {
                anyhow::bail!("不支持的 id 类型，只支持 ss123 和 ep123 类型");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type T = InputType;

    #[test]
    fn parse_bv() {
        assert_eq!(
            "https://www.bilibili.com/video/BV1z44y1E7m6"
                .parse::<T>()
                .unwrap(),
            T::BV {
                bv: "BV1z44y1E7m6".to_string(),
                p: None
            }
        );

        assert_eq!(
            "https://www.bilibili.com/video/BV1z44y1E7m6?p=2"
                .parse::<T>()
                .unwrap(),
            T::BV {
                bv: "BV1z44y1E7m6".to_string(),
                p: Some(2)
            }
        );

        assert_eq!(
            "BV1z44y1E7m6".parse::<T>().unwrap(),
            T::BV {
                bv: "BV1z44y1E7m6".to_string(),
                p: None
            }
        );
    }

    #[test]
    fn parse_season_or_episode() {
        assert_eq!(
            "https://www.bilibili.com/bangumi/play/ss28296"
                .parse::<T>()
                .unwrap(),
            T::Season { season_id: 28296 }
        );
        assert_eq!(
            "https://www.bilibili.com/bangumi/play/ep473502"
                .parse::<T>()
                .unwrap(),
            T::Episode { episode_id: 473502 }
        );

        assert_eq!(
            "ss28296".parse::<T>().unwrap(),
            T::Season { season_id: 28296 }
        );
        assert_eq!(
            "ep473502".parse::<T>().unwrap(),
            T::Episode { episode_id: 473502 }
        );
    }
}
