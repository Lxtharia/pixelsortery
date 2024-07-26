#![allow(unused_parens, unused)]
use image::{codecs::png::{self, PngEncoder}, ImageResult, Rgb, RgbImage};
use path_creator::PathCreator;
use span_sorter::{SortingCriteria, SpanSorter};
use std::{
    fs::File, path::Path, time::Instant
};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod path_creator;
pub mod pixel_selector;
pub mod span_sorter;

pub struct Pixelsorter {
    img: RgbImage,
    pub sorter: span_sorter::SpanSorter,
    pub selector: Box<dyn PixelSelector>,
    pub path_creator: path_creator::PathCreator,
    pub reverse: bool,
}

const BENCHMARK: bool = true;

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

    // 1:1 wrapper for image save function
    pub fn save<Q: AsRef<Path>>(&self, path: Q) -> ImageResult<()> {
        let f = File::create(path).expect("Error saving to file");
        let png_encoder = PngEncoder::new_with_quality(f, png::CompressionType::Best, png::FilterType::Adaptive);
        self.img.write_with_encoder(png_encoder)
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
        eprintln!("Image information: {} x {} ({} pixels)", self.img.width(), self.img.height(), pixelcount);

        eprintln!(
            "Sorting with:\n  > {}{}\n  > {}\n  > {}",
            self.path_creator.info_string(),
            if self.reverse {" [Reversed]"} else {""},
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        if BENCHMARK {
            timestart = Instant::now();
        }

        let ranges = self.path_creator.create_paths(&mut self.img, self.reverse);

        if BENCHMARK {
            eprintln!("TIME [Creating Paths]:\t{:?}", timestart.elapsed());
            timestart = Instant::now();
        }

        // CREATE SPANS
        let mut spans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        for r in ranges {
            for span in self.selector.create_spans(&mut r.into()) {
                spans.push(span);
            }
        }
        if BENCHMARK {
            eprintln!("TIME [Selector]:\t{:?}", timestart.elapsed());
            eprintln!("Amount of spans:\t{}", &spans.len());
            timestart = Instant::now();
        }

        // SORT EVERY SPAN
        for mut span in spans {
            self.sorter.sort(&mut span);
        }

        if BENCHMARK {
            let timeend = timestart.elapsed();
            eprintln!("TIME [Sorting]: \t{:?}", timeend);
        }
    }
}
