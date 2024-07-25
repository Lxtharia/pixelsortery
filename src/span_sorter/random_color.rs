use image::Rgb;
use rand::Rng;

pub fn set_random_color(pixels: &mut [&mut Rgb<u8>], _: for<'a> fn(&'a Rgb<u8>) -> u16 ) {
    // put them back at the pointer locations
    let mut rng = rand::thread_rng();
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
