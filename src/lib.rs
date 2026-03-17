#![allow(unused_parens, unused)]
use eframe::egui::TextBuffer;
use image::{GenericImage, GenericImageView, ImageResult, Rgb, RgbImage, RgbaImage, buffer::EnumeratePixels, codecs::png::PngEncoder};
use log::{debug, error, info, warn};
use path_creator::PathCreator;
use pixel_selector::get_criteria_function;
use rayon::prelude::*;
use span_sorter::{SortingCriteria, SpanSorter};
use std::{any::Any, collections::VecDeque, fmt::Debug, fs, io::{self, ErrorKind, Read, Write}, path::{Path, PathBuf}, process::{self, Command, Output, Stdio}, time::Instant};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod path_creator;
pub mod pixel_selector;
pub mod span_sorter;
#[cfg(feature = "video")]
mod video;

pub type Span = Vec<Rgb<u8>>;
pub type MutSpan<'a> = Vec<&'a mut Rgb<u8>>;
pub type MutSpanVec<'a> = Vec<Vec<&'a mut Rgb<u8>>>;
type CriteriaFunction = for<'a> fn(&'a Rgb<u8>) -> u16;

#[derive(Clone)]
pub struct Pixelsorter {
    pub sorter: span_sorter::SpanSorter,
    pub selector: PixelSelector,
    pub path_creator: path_creator::PathCreator,
    pub reverse: bool,
}

#[derive(Clone)]
pub(crate) struct PixelInfo {
    coords: (u32, u32),
    pixel: Rgb<u8>,
    select_value: u16,
    // sort_value: u64, // Probably smarter to calculate this when needed
}

impl PixelInfo {
    /// Returns a new PixelInfo with a different color
    fn with_pixel(&self, px: Rgb<u8> ) -> Self {
        PixelInfo {
            coords: self.coords,
            select_value: self.select_value,
            pixel: px,
        }
    }
}
trait ToPixel {
    #[inline]
    fn pixel(&self) -> &Rgb<u8>;
}
impl ToPixel for &PixelInfo {
    #[inline]
    fn pixel(&self) -> &Rgb<u8> {
        &self.pixel
    }
}
impl ToPixel for &mut Rgb<u8> {
    #[inline]
    fn pixel(&self) -> &Rgb<u8> {
        self
    }
}

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
    pub fn to_long_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "All Horizontally".into(),
            PathCreator::AllVertically => "All Vertically".into(),
            PathCreator::HorizontalLines => if self.reverse {"Left"} else {"Right"}.into(),
            PathCreator::VerticalLines => if self.reverse {"Up"} else {"Down"}.into(),
            PathCreator::Circles => "Circles".into(),
            PathCreator::Spiral => "Spiral".into(),
            PathCreator::SquareSpiral => "Square Spiral".into(),
            PathCreator::RectSpiral => "Rect Spiral".into(),
            PathCreator::Diagonally(a) => format!("Diagonally ({}°)", a),
            PathCreator::Hilbert => "Hilbert Curve".into(),
            p => format!("{}", p),
        }
        .as_str();
        s += "-";
        if self.reverse {
            s += "R-"
        };
        s += match self.selector {
            PixelSelector::Full => "Full".into(),
            PixelSelector::Fixed { len } => format!("Fixed length ({})", len),
            PixelSelector::Random { max } => format!("Random length ({})", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{}{}-{}",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "Hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "Brightness",
                    pixel_selector::PixelSelectCriteria::Saturation => "Saturation",
                },
                min,
                max
            ),
        }
        .as_str();
        s += "-";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "Mapsort",
            span_sorter::SortingAlgorithm::Shellsort => "Shellsort",
            span_sorter::SortingAlgorithm::Glitchsort => "Glitchsort",
            span_sorter::SortingAlgorithm::DebugColor => "Debug-colors",
        };
        s += "-";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "Hue",
            SortingCriteria::Brightness => "Brightness",
            SortingCriteria::Saturation => "Saturation",
        };

        s
    }

    pub fn to_pretty_short_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "Horizontal".into(),
            PathCreator::AllVertically => "Vertical".into(),
            PathCreator::HorizontalLines => if self.reverse {"Left"} else {"Right"}.into(),
            PathCreator::VerticalLines => if self.reverse {"Up"} else {"Down"}.into(),
            PathCreator::Circles => "Circles".into(),
            PathCreator::Spiral => "Spiral".into(),
            PathCreator::SquareSpiral => "Square".into(),
            PathCreator::RectSpiral => "Rect".into(),
            PathCreator::Diagonally(a) => format!("Diag({}°)", a),
            PathCreator::Hilbert => "Hilbert".into(),
            p => format!("{}", p),
        }
        .as_str();
        if self.reverse {
            s += "{R}"
        };
        s += " | ";

        s += match self.selector {
            PixelSelector::Full => "Full".into(),
            PixelSelector::Fixed { len } => format!("Fixed ({})", len),
            PixelSelector::Random { max } => format!("Random ({})", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{} ({}-{})",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "Hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "Bright",
                    pixel_selector::PixelSelectCriteria::Saturation => "Sat",
                },
                min,
                max
            ),
        }
        .as_str();
        s += " | ";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "Map",
            span_sorter::SortingAlgorithm::Shellsort => "Shell",
            span_sorter::SortingAlgorithm::Glitchsort => "Glitch",
            span_sorter::SortingAlgorithm::DebugColor => "Debug",
        };
        s += "(by ";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "Hue",
            SortingCriteria::Brightness => "Bright",
            SortingCriteria::Saturation => "Sat",
        };
        s += ")";

        s
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
            p => format!("{}", p).to_lowercase(),
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
        self.sorter.sort_mut(&mut pixels);
    }
    pub fn sort(&self, img: &mut RgbImage) {
        let (w, h) = (img.width().into(), img.height().into());
        self.sort_pixels(img.pixels_mut().collect(), w, h);
    }

    /// Sort a given image in place
    pub fn sort_pixels(&self, all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) {
        let mut timestart = Instant::now();
        // a vector containing pointers to each pixel
        let pixelcount = w * h;
        info!( "Image information: {} x {} ({} pixels)", w, h, pixelcount);

        info!(
            "Sorting with:\n   | {}{}\n   | {}\n   | {}",
            self.path_creator.info_string(),
            if self.reverse { " [Reversed]" } else { "" },
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        // CUT IMAGE INTO PATHS
        timestart = Instant::now();
        info!("TIME | [Loading pixels]: \t+ {:?}", timestart.elapsed());
        let ranges: Vec<Vec<&mut Rgb<u8>>> = self.path_creator.create_paths(all_pixels, w, h, self.reverse);

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
            self.sorter.sort_mut(&mut span);
        });

        let timeend = timestart.elapsed();
        info!("TIME [Sorting]: \t{:?}", timeend);
    }

    pub fn mask(&self, img: &mut RgbImage) -> bool {
        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        if let PixelSelector::Threshold { min, max, criteria } = self.selector {
            self.selector.mask_mut(&mut all_pixels);
            return true;
        }
        return false;
    }
}

pub struct CachedPixelsorter<'a> {
    image: RgbImage,
    pixels: Vec<PixelInfo>,
    paths: Option<Vec<Vec<&'a PixelInfo>>>,
    previous_opts: Option<Pixelsorter>,
}

impl<'a> CachedPixelsorter<'a> {
    pub fn new(image: RgbImage) -> Self {
        CachedPixelsorter {
            pixels: Vec::with_capacity((image.width() * image.height()) as usize),
            image,
            paths: None,
            previous_opts: None,
        }
    }


    pub fn sort(&'a mut self, options: &Pixelsorter) -> RgbImage {
        let w = self.image.width().into();
        let h = self.image.height().into();
        // Caching is difficult with this mutable setup, because we can only keep mutable references
        // let all_pixels: Vec<PixelInfo> = self.image.enumerate_pixels_mut();

        // FIRST COPY
        if self.pixels.is_empty() {
            self.pixels = self.image.enumerate_pixels().map(|ep|
                // match options.selector {
                //     PixelSelector::Threshold { min, max, criteria } => {
                //         let crit = get_criteria_function(criteria);
                //         PixelInfo { coords: (ep.0, ep.1), pixel: *ep.2, select_value: crit(ep.2) }
                //     }
                    // _ =>
                PixelInfo { coords: (ep.0, ep.1), pixel: *ep.2, select_value: 555 }
                // }
            ).collect();
        }
        // TODO: Cache the criteria value when the criteria changes

        let mut timestart = Instant::now();
        info!("TIME | [Loading pixels]: \t+ {:?}", timestart.elapsed());
        // CUT IMAGE INTO PATHS
        if self.paths.is_none() {
            timestart = Instant::now();
            self.paths = Some(options.path_creator.create_paths(
                // COPY REFERENCES
                self.pixels.iter().collect(),
                w, h, options.reverse
            ));
            info!("TIME [Creating Paths]:\t{:?}", timestart.elapsed());
        }

        timestart = Instant::now();
        // CREATE SPANS ON EVERY PATH
        let mut spans: Vec<Vec<&PixelInfo>> = Vec::new();
        spans.par_extend(
            self.paths
                .as_ref()
                .unwrap()
                .par_iter()
                .map(|r| options.selector.create_spans(&mut r.clone().into())) // CLONE REFERENCES
                .flatten(),
        );

        info!("TIME [Selector]:\t{:?}", timestart.elapsed());
        info!("Amount of spans:\t{}", &spans.len());
        timestart = Instant::now();


        // SORT EVERY SPAN
        let sorted_spans = spans
            .into_par_iter()
            .map(|mut span| {
                options.sorter.sort(&span)
            });
        info!("TIME [Sorting]: \t{:?}", timestart.elapsed());
        timestart = Instant::now();

        let mut sorted = self.image.clone();
        sorted_spans.flatten().collect::<Vec<PixelInfo>>().iter().for_each( |pi| {
            sorted.put_pixel(pi.coords.0, pi.coords.1, pi.pixel);
        });

        info!("TIME [Writing]: \t{:?}", timestart.elapsed());


        sorted
    }

    // TODO:
    // pub fn mask(&self, img: &mut RgbImage) -> bool {
    //     let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    //     if let PixelSelector::Threshold { min, max, criteria } = self.selector {
    //         self.selector.mask_mut(&mut all_pixels);
    //         return true;
    //     }
    //     return false;
    // }

}
