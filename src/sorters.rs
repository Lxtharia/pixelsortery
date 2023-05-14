mod mapsort;
mod random_color;

use image::{RgbImage, Rgb, Pixel};
use crate::color_helpers::*;
use mapsort::*;


#[derive(Debug, Clone, Copy)]
pub enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
    Debug,
}



pub fn sort_whole_image(img: &mut RgbImage, method: &SortingMethod){
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    mut_map_sort(&mut pixels, method);
}


pub fn sort_img(img: &mut RgbImage, method: &SortingMethod){
    let (width, height) = img.dimensions();
    // a vector of pointers to the pixels
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    
    let sorting_function = match method {
        SortingMethod::Debug => random_color::set_random_color,
        _ => mut_map_sort,
    };
    println!("{:?}", method);
    // We are iterating through all lines.
    // What if we want to iterate through pixels diagonally?
    // Or in a hilbert curve?
    // So we need an array of iterators (diagonal lines), or just one iterator
    // each iterator needs to have mutable pixel pointers we can write to
    let mut k = 0;
    let mut start = 0;
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            
            if get_hue(pixels[index]) >= 180 && get_brightness(pixels[index]) < 130 && index != (width*height) as usize { // valid pixel
                k+=1;
            } else { 
                if k> 0 { // if it's more than one pixel 
                    // we give mut_map_sort a mutable slice of RGB-pointers
                   sorting_function(&mut pixels[start..=start+k], method);
                }
                k = 0;
                start = 1+index;
            }
        }
    }

}

