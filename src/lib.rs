use std::path::Path;
use std::env;
use image::{ImageResult, Rgb, RgbImage};
pub mod sorter;
mod color_helpers;
use color_helpers::*;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: sorter::SpanSorter,
}


/*
Ah yes, notes:
This should be the basic interaction

ps = new(img) -> PS with default options
ps = new(img, Options{})
ps.sort()
ps.sort(Options{}) // This should override only set options

I want to animate/easily adjust paremeter & chain sorting calls (efficiently, so best to not copy image every time)
ps.options.min = 130
ps.options.max = 45

*/



impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage, sorter: sorter::SpanSorter) -> Pixelsorter {
        Pixelsorter { img, sorter, }
    }

    // 1:1 wrapper for image save function
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> ImageResult<()> {
        self.img.save(path)
    }

    // sorting without creating spans
    pub fn sort_all_pixels(&mut self){
        let mut pixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();
        self.sorter.sort(&mut pixels);
    }

    pub fn test(x: u8) -> u8 {
        let y = x +123891;
        return y
    }

    pub fn sort(&mut self){

        let (width, height) = self.img.dimensions();
        // a vector containing pointers to each pixel
        let mut pixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();
        let x;

        println!("Sorting with: {:?} ", self.sorter);

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
                       self.sorter.sort(&mut pixels[start..=start+k]);
                    }
                    k = 0;
                    start = 1+index;
                }
            }
        }
    }
}
