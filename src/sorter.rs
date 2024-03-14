use image::{RgbImage, Rgb, Pixel};
mod random_color;
mod mapsort;

#[derive(Debug)]
pub struct Sorter {
    pub method: SortingMethod,
    // alg: SortingAlgorithm,
    // inverse: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
    Debug,
}

impl Sorter {
    // Sort slice of pixels
    pub fn sort(&self, pixels: &mut [&mut Rgb<u8>]) {
        // 
        let sorting_function = match self.method {
            SortingMethod::Debug => random_color::set_random_color,
            _ => mapsort::mut_map_sort,
        };
        // call sorting function
        sorting_function(pixels, &self.method);
    }
}

