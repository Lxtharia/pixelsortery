use std::any::type_name;
use std::collections::VecDeque;

use crate::color_helpers::*;
use crate::Span;
use image::{Pixel, Rgb, RgbImage};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, Rng,
};

#[derive(Debug)]
pub struct ThresholdSelector {
    pub min: u64,
    pub max: u64,
    pub criteria: PixelSelectCriteria,
}

#[derive(Debug)]
pub struct RandomSelector {
    pub max: u32,
}

/// Key criteria which a (Threshold-)Selector should use as a key
#[derive(Debug, Clone, Copy)]
pub enum PixelSelectCriteria {
    Hue,
    Brightness,
    Saturation,
}

/// Returns a list of pixel spans
pub trait PixelSelector {
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>>;
    fn info_string<'a>(&'a self) -> String; // I bet this is no the rust way
}

impl PixelSelector for RandomSelector {
    fn info_string(&self) -> String {
        format!("Random Selector with max length: {})", self.max)
    }
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();
        let mut rng = thread_rng();
        let rng_range = Uniform::from(0..self.max as usize);

        let len = pixels.len();

        while !pixels.is_empty() {
            // Random amount of pixels we want to take
            let mut r = rng_range.sample(&mut rng);
            // Prevent out of bounds error
            if pixels.len() < r {
                r = pixels.len();
            }
            // Take r pixels and put into new span
            let mut span: Vec<&mut Rgb<u8>> = Vec::new();
            for i in 0..r {
                span.push(pixels.pop_front().unwrap());
            }
            // Append span to our list of spans
            spans.push(span);
        }
        spans
    }
}

impl PixelSelector for ThresholdSelector {
    fn info_string(&self) -> String {
        format!(
            "Selecting Pixels with: {} < {:?} < {})",
            self.min, self.criteria, self.max
        )
    }
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

        let value_function = match self.criteria {
            PixelSelectCriteria::Hue => get_hue,
            PixelSelectCriteria::Brightness => get_brightness,
            PixelSelectCriteria::Saturation => get_saturation,
        };

        let start = 0usize;
        let mut span: Vec<&mut Rgb<u8>> = Vec::new();
        for _ in 0..pixels.len() {
            let val = value_function(pixels.get(0).unwrap());
            let px = pixels.pop_front().unwrap();

            if (val as u64) >= self.min && (val as u64) <= self.max {
                // A valid pixel. Add to span
                span.push(px);
            } else {
                // A invalid pixel, close the span and create a new one
                // Only do that when the current span isn't empty anyway
                if (span.len() > 0) {
                    spans.push(span);
                    span = Vec::new();
                }
            }
        }
        spans.push(span);
        spans
    }
}
