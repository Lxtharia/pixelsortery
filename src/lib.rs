use image::{RgbImage, Rgb, Pixel};
pub mod sorters;
mod color_helpers;

pub struct Pixelsorter {
    pub img: RgbImage,
    pub method: sorters::SortingMethod,
//    sorter: sorters::Sorter,
}
