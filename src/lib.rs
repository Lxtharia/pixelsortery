#![allow(unused)]
use color_helpers::*;
use image::{GenericImageView, ImageResult, Rgb, RgbImage};
use pixel_selector::{RandomSelector};
use span_sorter::{SortingCriteria, SpanSorter};
use rand::{thread_rng, Rng};
use std::{any::{type_name, Any}, collections::VecDeque, path::Path, time::Instant};

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
        let w: u64 = self.img.width().into();
        let mut mutpixels: VecDeque<&mut Rgb<u8>> = self.img.pixels_mut().collect();

        println!("Sorting with: {:?} and {:?} ", self.selector.debug_info(), self.sorter);

        // We are iterating through all lines.
        // What if we want to iterate through pixels diagonally?
        // Or in a hilbert curve?
        // So we need an array of iterators (diagonal lines), or just one iterator
        // each iterator needs to have mutable pixel pointers we can write to
        // for section in self.iterator.yieldIterators(mutpixels) { ... this stuff below ... }
        let mut prespans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        let mut prespan: Vec<&mut Rgb<u8>> = Vec::new();
        for i in 0..mutpixels.len() {
            let px = mutpixels.pop_front().unwrap();

            // When last pixel in the line
            if i as u64 % w < w-1 {
                // A valid pixel. Add to span
                prespan.push(px);
            } else {
                // Add last pixel, push span and create a new one
                prespan.push(px);
                prespans.push(prespan);
                prespan = Vec::new();
            }
        }
        prespans.push(prespan);



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
            self.sorter.sort(&mut span);
        }
            let timeend = timestart.elapsed();
            println!("Time [Sort]: {:?}", timeend);
    }
}
