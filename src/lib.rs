#![allow(unused)]
use color_helpers::*;
use image::{GenericImageView, ImageResult, Rgb, RgbImage};
use iterator::ImageIterator;
use pixel_selector::{RandomSelector};
use span_sorter::{SortingCriteria, SpanSorter};
use rand::{thread_rng, Rng};
use std::{any::{type_name, Any}, collections::VecDeque, path::Path, time::Instant};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod pixel_selector;
pub mod span_sorter;
pub mod iterator;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: span_sorter::SpanSorter,
    pub selector: Box<dyn PixelSelector>,
    pub iterator: iterator::ImageIterator,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage) -> Pixelsorter {
        Pixelsorter {
            img,
            sorter: SpanSorter::new(SortingCriteria::Hue),
            selector: Box::new(RandomSelector { max: 40 }),
            iterator: ImageIterator::All,
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

        println!("Sorting with: {:?} and {:?} ", self.selector.debug_info(), self.sorter);

        // We are iterating through all lines.
        // What if we want to iterate through pixels diagonally?
        // Or in a hilbert curve?
        // So we need an array of iterators (diagonal lines), or just one iterator
        // each iterator needs to have mutable pixel pointers we can write to
        // for section in self.iterator.yieldIterators(mutpixels) { ... this stuff below ... }
        let mut prespans = self.iterator.traverse(&mut self.img);

        //Still very slow dividing of all pixels into spans
            let timestart = Instant::now();
        // CREATE SPANS
            let mut mutspans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        for prespan in prespans {
            for span in self.selector.mutspans(&mut prespan.into()) {
                mutspans.push(span);
            }
        }
            let timeend = timestart.elapsed();
            println!("Time [Selector]: {:?}", timeend);
            println!("Amount of pixels: {}", pixelcount);
            println!("Amount of spans: {}", &mutspans.len());
            let timestart = Instant::now();
        // SORT EVERY SPAN
        for mut span in mutspans {
            // Only sort if there is at least 2 pixels in there
            if (span.len() > 1) {
                self.sorter.sort(&mut span);
            }
        }
            let timeend = timestart.elapsed();
            println!("Time [Sort]: {:?}", timeend);
    }
}
