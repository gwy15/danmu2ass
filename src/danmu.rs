//! 一个弹幕实例，但是没有位置信息
use super::CanvasConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DanmuType {
    Float,
    Top,
    Bottom,
    Reverse,
}
impl Default for DanmuType {
    fn default() -> Self {
        DanmuType::Float
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Danmu {
    pub timeline_s: f64,
    pub content: String,
    pub r#type: DanmuType,
    pub fontsize: u32,
    pub rgb: (u8, u8, u8),
}

impl Danmu {
    /// 计算弹幕的“像素长度”，会乘上一个缩放因子
    ///
    /// 汉字算一个全宽，英文算2/3宽
    pub fn length(&self, config: &CanvasConfig) -> f64 {
        let pts = self.fontsize
            * self
                .content
                .chars()
                .map(|ch| if ch.is_ascii() { 2 } else { 3 })
                .sum::<u32>()
            / 3;

        pts as f64 * config.width_ratio
    }
}
