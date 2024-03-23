#![allow(unused)]
use std::path::Path;
use image::{ImageResult, Rgb, RgbImage};
use crate::pixel_selector::PixelSelector;
use color_helpers::*;

mod color_helpers;
pub mod pixel_selector;
pub mod sorter;


pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: sorter::Sorter,
    selector: pixel_selector::RandomSelector,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage, sorter: sorter::Sorter) -> Pixelsorter {
        let random_selector = pixel_selector::RandomSelector{length: 40};
        Pixelsorter { img, sorter, selector: random_selector}
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

    pub fn sort(&mut self){

        let (width, height) = self.img.dimensions();
        // a vector containing pointers to each pixel
        let mut pixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();

        println!("Sorting with: {:?} ", self.sorter);

        // We are iterating through all lines.
        // What if we want to iterate through pixels diagonally?
        // Or in a hilbert curve?
        // So we need an array of iterators (diagonal lines), or just one iterator
        // each iterator needs to have mutable pixel pointers we can write to
        let mut k = 0;
        let mut start = 0;
        let spans = self.selector.spans(&pixels);
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
