use image::Rgb;
use rand::{rngs::*, Rng, SeedableRng};
use crate::CriteriaFunction;

use crate::PixelInfo;

pub fn set_random_color_mut(pixels: &mut [&mut Rgb<u8>], _: CriteriaFunction) {
    // put them back at the pointer locations
    let mut rng = rand::thread_rng();
    //let mut rng = StdRng::seed_from_u64(pixels.get(0).and_then(|p| Some(p.0[0] as u64)).unwrap_or(123198412));
    let ran_col = Rgb {
        0: [
            rng.gen_range(80..=240),
            rng.gen_range(80..=240),
            rng.gen_range(80..=240),
        ],
    };
    for p in pixels {
        **(p) = ran_col;
    }
}

pub fn set_random_color(pixels: &[&PixelInfo], _: CriteriaFunction) -> Vec<PixelInfo> {
    // put them back at the pointer locations
    let mut rng = rand::thread_rng();
    //let mut rng = StdRng::seed_from_u64(pixels.get(0).and_then(|p| Some(p.0[0] as u64)).unwrap_or(123198412));
    let ran_col = Rgb {
        0: [
            rng.gen_range(80..=240),
            rng.gen_range(80..=240),
            rng.gen_range(80..=240),
        ],
    };

    pixels.iter().map(|pi| pi.with_pixel(ran_col)).collect()
}
