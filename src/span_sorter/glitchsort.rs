use std::mem;

use super::SortingCriteria;
use crate::color_helpers::*;
use ::array_init::array_init;
use image::{Rgb, RgbImage};

#[derive(Debug)]
struct PixelWrapper {
    ind: usize,
    val: u16,
}

fn glitch_swap(pixels: &mut [&mut Rgb<u8>], wrapper_vec: &mut Vec<PixelWrapper>, x: usize, y: usize){
    // "Just" swap the Wrapper Objects in the Array and then change the color of the objects pixels

    let i = wrapper_vec[x].ind;
    let j = wrapper_vec[y].ind;

    // print!("  Swap: {:?} ({}) <-> {:?} ({})", pixels[i], wrapper_vec[x].val,pixels[j], wrapper_vec[y].val);

    wrapper_vec.swap(x,y);
    let tmp = pixels[i].clone() ;
    *pixels[i] = pixels[j].clone();
    *pixels[j] = tmp;

    // This does not work:
    // pixels.swap(i, j);

    // The glitch comes from the fact that
    // We switch wrap_x and wrap_y and then swap their colors
    // But they still point to their initial index
}


pub fn glitchsort_mut(pixels: &mut [&mut Rgb<u8>], value_function: for<'a> fn(&'a Rgb<u8>) -> u16){
    let span_len = pixels.len();
    let mut wrappers = Vec::new();

    for i in 0..pixels.len() {
        let val = value_function(pixels[i]);
        wrappers.push(PixelWrapper{ind: i, val});
    }

    println!("New span");
    let mut gap = span_len;
    let mut swapped = false;
    while ( (gap > 1) || swapped ) {
        if (gap > 1){ gap = (gap as f64/1.247330950103979) as usize; }
        swapped = false;
        for i in 0..span_len {
            if (gap + i >= span_len){break;}
            if ( (wrappers[i+gap].val as i8) > wrappers[i].val as i8 ){
                glitch_swap(pixels, &mut wrappers, i+gap, i);
                println!();
                swapped = true;
            }
        }
    }

}

