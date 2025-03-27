use image::Rgb;
use rand::{rngs::*, Rng, SeedableRng};

pub fn set_random_color(pixels: &mut [&mut Rgb<u8>], _: for<'a> fn(&'a Rgb<u8>) -> u16 ) {
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
