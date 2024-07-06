use std::collections::VecDeque;

use crate::color_helpers::*;
use crate::Span;
use image::{Pixel, Rgb, RgbImage};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, Rng,
};

#[derive(Debug, Clone, Copy)]
pub enum PixelSelectCriteria {
    Hue,
    Brightness,
    Saturation,
}

pub trait PixelSelector {
    /// Returns a list of pixel spans
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>>;
}

#[derive(Debug)]
pub struct ThresholdSelector {
    min: u64,
    max: u64,
    criteria: PixelSelectCriteria,
}

#[derive(Debug)]
pub struct RandomSelector {
    pub max: i32,
}

impl PixelSelector for RandomSelector {
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();
        let mut rng = thread_rng();
        let rng_range = Uniform::from(0..40);

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
            let val = get_hue(pixels.get(0).unwrap());
            let px = pixels.pop_front().unwrap();

            if val as u64 >= self.min && val as u64 <= self.max {
                // A valid pixel. Add to span
                span.push(px);
            } else {
                // A invalid pixel, close the span and create a new one
                spans.push(span);
                span = Vec::new();
            }
        }
        spans
    }
}
