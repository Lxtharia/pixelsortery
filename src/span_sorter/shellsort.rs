use super::SortingCriteria;
use crate::color_helpers::*;
use ::array_init::array_init;
use image::{Rgb, RgbImage};

#[derive(Debug)]
struct PixelWrapper{
    px:  Rgb<u8>,
    val: u16,
}

pub fn shellsort_mut(pixels: &mut [&mut Rgb<u8>], method: &SortingCriteria){
    // Stolen from some Stackoverflow Thread
    use SortingCriteria::*;
    let value_function = match method {
        Brightness => get_brightness,
        Saturation => get_saturation,
        Hue | _ => get_hue,
    };

    let span_len = pixels.len();
    let mut fake_pixels = Vec::new();
    // Wrap each pixel into a wrapper with a calculated value (TODO: from SortingCriteria)
    pixels.into_iter().for_each(|px| {
        let val = value_function(px);
        fake_pixels.push(PixelWrapper{px: **px, val});
    });

    let mut gap = span_len;
    let mut swapped = false;
    while ( (gap > 1) || swapped ) {
        if (gap > 1){ gap = (gap as f64/1.247330950103979) as usize; }
        swapped = false;
        for i in 0..span_len {
            if (gap + i >= span_len){break;}
            if ( fake_pixels[i+gap].val < fake_pixels[i].val ){
                fake_pixels.swap(i+gap, i);
                swapped = true;
            }
        }
    }

    for i in 0..pixels.len() {
        *pixels[i] = fake_pixels[i].px;
    }

}

