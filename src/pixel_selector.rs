use crate::color_helpers::*;
use image::Rgb;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelDrainRange,
    ParallelIterator,
};
use std::{cmp::min, collections::VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSelector {
    Full,
    Fixed {
        len: u64,
    },
    Random {
        max: u32,
    },
    Threshold {
        min: u64,
        max: u64,
        criteria: PixelSelectCriteria,
    },
}

/// Key criteria which a (Threshold-)Selector should use as a key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSelectCriteria {
    Hue,
    Brightness,
    Saturation,
}

impl PixelSelector {
    /// Returns a list of pixel spans
    pub fn create_spans<'a>(
        self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>> {
        match self {
            PixelSelector::Full => full_selector(pixels),
            PixelSelector::Fixed { len } => fixed_selector(pixels, len),
            PixelSelector::Random { max } => random_selector(pixels, max),
            PixelSelector::Threshold { min, max, criteria } => threshold_selector(pixels, criteria, min, max),
        }
    }
    pub fn info_string<'a>(self) -> String {
        match self {
            PixelSelector::Full => String::from("Selecing all pixels"),
            PixelSelector::Fixed { len } => format!("Selecting ranges of fixed length {}", len),
            PixelSelector::Random { max } => format!("Random Selector with max length {}", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "Selecting Pixels with: [{} < {:?} < {}]",
                min, criteria, max
            ),
        }
    }
}

fn full_selector<'a>(pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>> {
    let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

    let mut span: Vec<&mut Rgb<u8>> = Vec::new();
    while !pixels.is_empty() {
        span.push(pixels.pop_front().unwrap());
    }
    spans.push(span);
    spans
}

fn fixed_selector<'a>(
    pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    len: u64,
) -> Vec<Vec<&'a mut Rgb<u8>>> {
    let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

    // Prevent an endless loop
    if len == 0 {
        return spans;
    }

    while pixels.len() >= len as usize {
        // Take len pixels and put into new span
        spans.push(pixels.drain(0..len as usize).collect());
    }
    // Push the rest
    spans.push(pixels.drain(..).collect());

    spans
}

fn random_selector<'a>(
    pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    max: u32,
) -> Vec<Vec<&'a mut Rgb<u8>>> {
    let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();
    // rng_range cannot be 1..1
    if max <= 1 {
        return spans;
    }
    let mut rng = thread_rng();
    let rng_range = Uniform::from(1..max as usize);

    loop {
        let mut r = rng_range.sample(&mut rng);
        if pixels.len() < r {
            break;
        }
        // Take r pixels and put into new span
        spans.push(pixels.drain(0..r).collect());
    }
    // Push the rest
    spans.push(pixels.drain(..).collect());

    spans
}

fn threshold_selector<'a>(
    pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    criteria: PixelSelectCriteria,
    min: u64,
    max: u64,
) -> Vec<Vec<&'a mut Rgb<u8>>> {
    let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

    let value_function = match criteria {
        PixelSelectCriteria::Hue => get_hue,
        PixelSelectCriteria::Brightness => get_brightness,
        PixelSelectCriteria::Saturation => get_saturation,
    };

    // Function that checks if a value is valid
    let valid = |val| (val as u64) >= min && (val as u64) <= max;

    let mut span: Vec<&mut Rgb<u8>> = Vec::new();
    for _ in 0..pixels.len() {
        let value = value_function(pixels.get(0).unwrap());
        let px = pixels.pop_front().unwrap();

        if valid(value) {
            // A valid pixel. Add to span
            span.push(px);
        } else {
            // A invalid pixel, close the span and create a new one
            // Only do that when the current span isn't empty anyway
            if span.len() > 0 {
                spans.push(span);
                span = Vec::new();
            }
        }
    }
    spans.push(span);
    spans
}
