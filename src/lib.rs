use std::path::Path;

use image::{ImageResult, RgbImage};
pub mod sorters;
mod color_helpers;

pub struct Pixelsorter {
    img: RgbImage,
    pub method: sorters::SortingMethod,
//    sorter: sorters::Sorter,
}

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage, method: sorters::SortingMethod ) -> Pixelsorter {
        Pixelsorter {
            img,
            method,
        }
    }

    // 1:1 wrapper for image save function
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> ImageResult<()> {
        self.img.save(path)
    }
}
