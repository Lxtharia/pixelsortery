#![allow(unused)]

use color_helpers::*;
use image::{GenericImageView, ImageResult, Rgb, RgbImage};
use iterator::ImageIterator;
use pixel_selector::RandomSelector;
use rand::{thread_rng, Rng};
use span_sorter::{SortingCriteria, SpanSorter};
use std::{
    any::{type_name, Any},
    collections::VecDeque,
    path::Path,
    time::Instant,
};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod iterator;
pub mod pixel_selector;
pub mod span_sorter;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: span_sorter::SpanSorter,
    pub selector: Box<dyn PixelSelector>,
    pub iterator: iterator::ImageIterator,
    pub reverse: bool,
}

pub type Span = Vec<Rgb<u8>>;

const BENCHMARK: bool = false;

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage) -> Pixelsorter {
        Pixelsorter {
            img,
            sorter: SpanSorter::new(SortingCriteria::Hue),
            selector: Box::new(RandomSelector { max: 40 }),
            iterator: ImageIterator::All,
            reverse: false
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
        let (mut timestart, mut timeend) = (Instant::now(), 0);
        // a vector containing pointers to each pixel
        let pixelcount = self.img.width() * self.img.height();

        println!(
            "Sorting with:\n\t{}\n\t{}\n\t{}",
            self.iterator.info_string(),
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        if (BENCHMARK) {
            let timestart = Instant::now();
        }

        let mut ranges = self.iterator.traverse(&mut self.img, self.reverse);

        if (BENCHMARK) {
            let timeend = timestart.elapsed();
            println!("Time [Selector]: {:?}", timeend);
            let timestart = Instant::now();
        }

        // CREATE SPANS
        let mut spans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        for r in ranges {
            for span in self.selector.mutspans(&mut r.into()) {
                spans.push(span);
            }
        }
        if (BENCHMARK) {
            let timeend = timestart.elapsed();
            println!("Time [Selector]: {:?}", timeend);
            println!("Amount of pixels: {}", pixelcount);
            println!("Amount of spans: {}", &spans.len());
            let timestart = Instant::now();
        }

        // SORT EVERY SPAN
        for mut span in spans {
            // Only sort if there is at least 2 pixels in there
            if (span.len() > 1) {
                self.sorter.sort(&mut span);
            }
        }

        if (BENCHMARK) {
            let timeend = timestart.elapsed();
            println!("Time [Sort]: {:?}", timeend);
        }
    }
}
