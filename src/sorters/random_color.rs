use ::array_init::array_init;
use image::{RgbImage, Rgb, Pixel};
use crate::sorters::SortingMethod;
use rand::Rng;

pub fn set_random_color(pixels: &mut [&mut Rgb<u8>], method: &SortingMethod) {
    // put them back at the pointer locations
    let mut rng = rand::thread_rng();
    let ran_col = Rgb {0: [rng.gen_range(130..=255), rng.gen_range(130..=255), rng.gen_range(130..=255)]};
    for pp in pixels {
        **(pp) = ran_col; 
    }
}


