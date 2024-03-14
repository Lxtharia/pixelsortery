use std::path::Path;

use image::{ImageResult, RgbImage};
pub mod sorter;
mod color_helpers;

pub struct Pixelsorter {
    img: RgbImage,
    pub method: sorter::SortingMethod,
//    sorter: sorters::Sorter,
}

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage, method: sorter::SortingMethod ) -> Pixelsorter {
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
