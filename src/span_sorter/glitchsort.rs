use super::SortingCriteria;
use crate::color_helpers::*;
use ::array_init::array_init;
use image::{Rgb, RgbImage};

#[derive(Debug)]
struct PixelWrapper{
    px:  Rgb<u8>,
    val: u16,
}

fn glitch_swap(wrapper_vec: &mut Vec<PixelWrapper>, x: usize, y: usize){
    // "Just" swap the Wrapper Objects in the Array and then change the color of the objects pixels
    wrapper_vec.swap(x,y);

    let temp_val = wrapper_vec[x].px;
    wrapper_vec[x].px = wrapper_vec[y].px;
    wrapper_vec[y].px = temp_val;

    // println!(">After: {:?} and {:?}", wrapper_vec[x], wrapper_vec[y]);

    // The glitch comes from the fact that
    // We switch wrap_x and wrap_y and then swap their colors
    // But the value stays the same, which we use to further sort

	// x is pointing to the pixel/color cx, y to cy
	// we now swap x and y, because we sort these and they have to pull the actual color struct themselves
	// what we do here is. We swap the pixels cx and cy. This puts these in the right order. 
	// But x is still pointing to the previous address where cy is now located.
	// So not only is the hue inaccurate to the pixel, if x were to swap with z, x would swap cy cause thats the pixel it points to 
	// I don't know how to write efficient comments, but this will help me understand when i forget again
}

pub fn glitchsort_mut(pixels: &mut [&mut Rgb<u8>], value_function: for<'a> fn(&'a Rgb<u8>) -> u16){
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
                glitch_swap(&mut fake_pixels, i+gap, i);
                swapped = true;
            }
        }
    }

    for i in 0..pixels.len() {
        *pixels[i] = fake_pixels[i].px;
    }

}

