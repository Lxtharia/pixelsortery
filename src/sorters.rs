mod mapsort;

use image::{RgbImage, Rgb, Pixel};
use crate::color_helpers;
use mapsort::*;


#[derive(Debug, Clone, Copy)]
pub enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
}



pub fn sort_whole_image(img: &mut RgbImage, method: &SortingMethod){
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    mut_map_sort(&mut pixels, method);
}


pub fn sort_img(img: &mut RgbImage, method: &SortingMethod){
    let (width, height) = img.dimensions();
    // a vector of pointers to the pixels
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    // We are iterating through all lines.
    // What if we want to iterate through pixels diagonally?
    // Or in a hilbert curve?
    // So we need an array of iterators (diagonal lines), or just one iterator
    // each iterator needs to have mutable pixel pointers we can write to
    let mut k = 0;
    let mut start = 0;
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;

            if k>=2 || x >= width-1 {
                // we give mut_map_sort a mutable slice of RGB-pointers
                mut_map_sort(&mut pixels[start..start+k], method);
                start = 1+index as usize ;
                k = 0;
            } else {
                k+=1;
            }
        }
    }

}

