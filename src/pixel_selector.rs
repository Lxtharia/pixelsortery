use image::{RgbImage, Rgb, Pixel};
use rand::{distributions::{Distribution, Uniform}, thread_rng, Rng};
use crate::Span;

pub trait PixelSelector {
    /// Returns a list of pixel spans
    fn spans<'a>(&'a self, pixels: &Vec<&'a Rgb<u8>>) -> Vec<Vec<&Rgb<u8>>>;
    fn mutspans<'a>(&'a self, pixels: &mut Vec<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>>;
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
            let mut end = i+r as usize;
            if end > len { end = len; }
            let mut span = pixels[i..=end].to_vec();
            spans.push(span);
            i += r + 1;
        }
        spans
    }

    fn mutspans<'a>(&'a self, pixels: &mut Vec<&'a mut Rgb<u8>>) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();
        let mut rng = thread_rng();
        let range = Uniform::from(0..40);

        while ! pixels.is_empty() {
            // Random amount of pixels we want to take
            let mut r = range.sample(&mut rng);
            // Take r pixels. If we have less then r pixels left, take all
            let mut span: Vec<&mut Rgb<u8>> = match pixels.len() >= r {
                true => pixels.drain(0..r).collect(),
                false => pixels.drain(0..).collect(),
            };
            // Append span to our list of spans
            spans.push(span);
        }
        spans
    }
}
