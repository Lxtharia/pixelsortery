#![allow(unused_parens, unused)]
use eframe::egui::TextBuffer;
use image::{codecs::png::PngEncoder, ImageResult, Rgb, RgbImage};
use log::{error, info, warn};
use path_creator::PathCreator;
use rayon::prelude::*;
use span_sorter::{SortingCriteria, SpanSorter};
use std::{path::Path, time::Instant};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod path_creator;
pub mod pixel_selector;
pub mod span_sorter;

#[derive(Clone)]
pub struct Pixelsorter {
    pub sorter: span_sorter::SpanSorter,
    pub selector: PixelSelector,
    pub path_creator: path_creator::PathCreator,
    pub reverse: bool,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new() -> Pixelsorter {
        Pixelsorter {
            sorter: SpanSorter::new(SortingCriteria::Brightness),
            selector: PixelSelector::Full,
            path_creator: PathCreator::AllHorizontally,
            reverse: false,
        }
    }

    pub fn to_compact_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "hor".into(),
            PathCreator::AllVertically => "vert".into(),
            PathCreator::HorizontalLines => "lr".into(),
            PathCreator::VerticalLines => "ud".into(),
            PathCreator::Circles => "circ".into(),
            PathCreator::Spiral => "sprl".into(),
            PathCreator::SquareSpiral => "spSq".into(),
            PathCreator::RectSpiral => "spRe".into(),
            PathCreator::Diagonally(a) => format!("diag{}", a),
            PathCreator::Hilbert => "hilb".into(),
        }
        .as_str();
        s += "-";
        if self.reverse {
            s += "R-"
        };
        s += match self.selector {
            PixelSelector::Full => "full".into(),
            PixelSelector::Fixed { len } => format!("fixed{}", len),
            PixelSelector::Random { max } => format!("rand{}", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{}{}-{}",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "bright",
                    pixel_selector::PixelSelectCriteria::Saturation => "sat",
                },
                min,
                max
            ),
        }
        .as_str();
        s += "-";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "map",
            span_sorter::SortingAlgorithm::Shellsort => "shell",
            span_sorter::SortingAlgorithm::Glitchsort => "gl",
            span_sorter::SortingAlgorithm::DebugColor => "debug",
        };
        s += "-";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "hue",
            SortingCriteria::Brightness => "bright",
            SortingCriteria::Saturation => "sat",
        };

        s
    }

    // sorting without creating spans
    pub fn sort_all_pixels(&self, img: &mut RgbImage) {
        let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        self.sorter.sort(&mut pixels);
    }

    /// Sort a given image in place
    pub fn sort(&self, img: &mut RgbImage) {
        let mut timestart = Instant::now();
        // a vector containing pointers to each pixel
        let pixelcount = img.width() * img.height();
        info!(
            "Image information: {} x {} ({} pixels)",
            img.width(),
            img.height(),
            pixelcount
        );

        info!(
            "Sorting with:\n   | {}{}\n   | {}\n   | {}",
            self.path_creator.info_string(),
            if self.reverse { " [Reversed]" } else { "" },
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        timestart = Instant::now();
        // CUT IMAGE INTO PATHS
        let ranges = self.path_creator.create_paths(img, self.reverse);

        info!("TIME [Creating Paths]:\t{:?}", timestart.elapsed());
        timestart = Instant::now();

        // CREATE SPANS ON EVERY PATH
        let mut spans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        spans.par_extend(
            ranges
                .into_par_iter()
                .map(|r| self.selector.create_spans(&mut r.into()))
                .flatten(),
        );

        info!("TIME [Selector]:\t{:?}", timestart.elapsed());

        info!("Amount of spans:\t{}", &spans.len());
        timestart = Instant::now();

        // SORT EVERY SPAN
        spans.into_par_iter().for_each(|mut span| {
            self.sorter.sort(&mut span);
        });

        let timeend = timestart.elapsed();
        info!("TIME [Sorting]: \t{:?}", timeend);
    }

    pub fn mask(&self, img: &mut RgbImage) -> Result<(), ()> {
        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        if let PixelSelector::Threshold { min, max, criteria } = self.selector {
            self.selector.mask(&mut all_pixels);
            return Ok(());
        }
        return Err(());
    }
}
