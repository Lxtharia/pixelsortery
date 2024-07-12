#![allow(unused)]
use color_helpers::*;
use image::{ImageResult, Rgb, RgbImage};
use pixel_selector::{RandomSelector};
use span_sorter::{SortingCriteria, SpanSorter};
use rand::{thread_rng, Rng};
use std::{any::{type_name, Any}, path::Path, time::Instant};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod pixel_selector;
pub mod span_sorter;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: span_sorter::SpanSorter,
    pub selector: Box<dyn PixelSelector>,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage) -> Pixelsorter {
        Pixelsorter {
            img,
            sorter: SpanSorter::new(SortingCriteria::Hue),
            selector: Box::new(RandomSelector { max: 40 }),
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
        let pixelcount= self.img.width() * self.img.height();
        let mut mutpixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();

        println!("Sorting with: {:?} and {:?} ", self.selector.debug_info(), self.sorter);

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
        println!("Amount of pixels: {}", pixelcount);
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
