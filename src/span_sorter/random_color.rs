use super::SortingCriteria;
use image::Rgb;
use rand::Rng;

pub fn set_random_color(pixels: &mut [&mut Rgb<u8>], _: &SortingCriteria) {
    // put them back at the pointer locations
    let mut rng = rand::thread_rng();
    let ran_col = Rgb {
        0: [
            rng.gen_range(130..=255),
            rng.gen_range(130..=255),
            rng.gen_range(130..=255),
        ],
    };
    for pp in pixels {
        **(pp) = ran_col;
    }
}
