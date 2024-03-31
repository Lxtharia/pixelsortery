#![allow(unused)]
use crate::pixel_selector::PixelSelector;
use color_helpers::*;
use image::{ImageResult, Rgb, RgbImage};
use rand::{thread_rng, Rng};
use std::{path::Path, time::Instant};

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
        let random_selector = pixel_selector::RandomSelector { max: 40 };
        Pixelsorter {
            img,
            sorter,
            selector: random_selector,
        }
    }

    // 1:1 wrapper for image save function
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> ImageResult<()> {
        self.img.save(path)
    }

    // sorting without creating spans
    pub fn sort_all_pixels(&mut self) {
        let mut pixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();
        self.sorter.sort(&mut pixels);
    }

    pub fn sort(&mut self) {
        // a vector containing pointers to each pixel
        let mut mutpixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();

        println!("Sorting with: {:?} and {:?} ", self.selector, self.sorter);

        // We are iterating through all lines.
        // What if we want to iterate through pixels diagonally?
        // Or in a hilbert curve?
        // So we need an array of iterators (diagonal lines), or just one iterator
        // each iterator needs to have mutable pixel pointers we can write to
        // for section in self.iterator.yieldIterators(mutpixels) { ... this stuff below ... }

        //Still very slow dividing of all pixels into spans
        let timestart = Instant::now();
        let mutspans = self.selector.mutspans(&mut mutpixels.into());
        let timeend = timestart.elapsed();
        println!("Time [Selector]: {:?}", timeend);
        println!("Amount of spans: {}", &mutspans.len());

        let timestart = Instant::now();
        // Sort every span
        for mut span in mutspans {
            self.sorter.sort(&mut span);
        }
        let timeend = timestart.elapsed();
        println!("Time [Sort]: {:?}", timeend);
    }
}
