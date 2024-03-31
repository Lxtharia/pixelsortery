use std::collections::VecDeque;

use crate::Span;
use image::{Pixel, Rgb, RgbImage};
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng, Rng,
};

pub trait PixelSelector {
    /// Returns a list of pixel spans
    fn spans<'a>(&'a self, pixels: &Vec<&'a Rgb<u8>>) -> Vec<Vec<&Rgb<u8>>>;
    fn mutspans<'a>(&'a self, pixels: &mut VecDeque<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>>;
}

#[derive(Debug)]
pub struct ThresholdSelector {
    min: f64,
    max: f64,
}

#[derive(Debug)]
pub struct RandomSelector {
    pub max: i32,
}

impl PixelSelector for RandomSelector {
    fn spans<'a>(&'a self, pixels: &Vec<&'a Rgb<u8>>) -> Vec<Vec<&Rgb<u8>>> {
        let mut spans: Vec<Vec<&Rgb<u8>>> = Vec::new();
        let len = pixels.len() - 1;
        let mut rng = thread_rng();
        let mut i = 0usize;
        while i < len {
            let r = rng.gen_range(0..self.max) as usize;
            let mut end = i + r as usize;
            if end > len {
                end = len;
            }
            let mut span = pixels.to_vec();
            spans.push(span);
            i += r + 1;
        }
        spans
    }

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
