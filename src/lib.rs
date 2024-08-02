#![allow(unused_parens, unused)]
use image::{codecs::png::PngEncoder, ImageResult, Rgb, RgbImage};
use log::{info, warn, error};
use path_creator::PathCreator;
use span_sorter::{SortingCriteria, SpanSorter};
use std::{
    path::Path,
    time::Instant,
};
use rayon::prelude::*;

use crate::pixel_selector::PixelSelector;


mod color_helpers;
pub mod path_creator;
pub mod pixel_selector;
pub mod span_sorter;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: span_sorter::SpanSorter,
    pub selector: Box<dyn PixelSelector + Sync>,
    pub path_creator: path_creator::PathCreator,
    pub reverse: bool,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new(img: RgbImage) -> Pixelsorter {
        Pixelsorter {
            img,
            sorter: SpanSorter::new(SortingCriteria::Brightness),
            selector: Box::new(pixel_selector::FullSelector{}),
            path_creator: PathCreator::AllHorizontally,
            reverse: false
        }
    }

    pub fn get_img(&self) -> RgbImage {
        self.img.clone()
    }
    // 1:1 wrapper for image save function
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> ImageResult<()> {
        self.img.save(path)
    }

    pub fn save_to_stdout(&self) -> ImageResult<()> {
        self.img.write_with_encoder(PngEncoder::new(std::io::stdout()))
    }

    // sorting without creating spans
    pub fn sort_all_pixels(&mut self) {
        let mut pixels: Vec<&mut Rgb<u8>> = self.img.pixels_mut().collect();
        self.sorter.sort(&mut pixels);
    }

    pub fn sort(&mut self) {
        let mut timestart = Instant::now();
        // a vector containing pointers to each pixel
        let pixelcount = self.img.width() * self.img.height();
        info!("Image information: {} x {} ({} pixels)", self.img.width(), self.img.height(), pixelcount);

        info!(
            "Sorting with:\n   | {}{}\n   | {}\n   | {}",
            self.path_creator.info_string(),
            if self.reverse {" [Reversed]"} else {""},
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        timestart = Instant::now();
        // CUT IMAGE INTO PATHS
        let ranges = self.path_creator.create_paths(&mut self.img, self.reverse);

        info!("TIME [Creating Paths]:\t{:?}", timestart.elapsed());
        timestart = Instant::now();

        // CREATE SPANS ON EVERY PATH
        let mut spans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        let span_iter = ranges.into_par_iter().map(|r| {
            self.selector.create_spans(&mut r.into())
        });

        info!("TIME [Selector]:\t{:?}", timestart.elapsed());
        spans = span_iter.flatten().collect();
        info!("TIME [Selector2]:\t{:?}", timestart.elapsed());

        info!("Amount of spans:\t{}", &spans.len());
        timestart = Instant::now();

        // SORT EVERY SPAN
        spans.into_par_iter().for_each(|mut span| {
            self.sorter.sort(&mut span);
        });

        let timeend = timestart.elapsed();
        info!("TIME [Sorting]: \t{:?}", timeend);
    }
}
