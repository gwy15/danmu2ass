#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;

mod ass_writer;
pub mod bilibili;
mod canvas;
mod cli;
mod danmu;
mod drawable;
mod input_type;
mod xml_parser;

pub use ass_writer::AssWriter;
pub use canvas::{Canvas, Config as CanvasConfig};
pub use cli::{Args, convert};
pub use danmu::Danmu;
pub use drawable::{DrawEffect, Drawable};
pub use input_type::InputType;
pub use xml_parser::Parser;
