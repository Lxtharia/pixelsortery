use image::{RgbImage, Rgb, Pixel};
use rand::{thread_rng, Rng};
use crate::Span;

pub trait PixelSelector {
    /// Returns a list of pixel spans
    fn spans<'a>(&'a self, pixels: &Vec<&'a Rgb<u8>>) -> Vec<Vec<&Rgb<u8>>>;
}

#[derive(Debug)]
pub struct ThresholdSelector {
    min: f64,
    max: f64,
}

#[derive(Debug)]
pub struct RandomSelector {
    pub length: i32,
}

impl PixelSelector for RandomSelector {
    fn spans<'a>(&'a self, pixels: &Vec<&'a Rgb<u8>>) -> Vec<Vec<&Rgb<u8>>> {
        let mut spans: Vec<Vec<&Rgb<u8>>> = Vec::new();
        let len = pixels.len() - 1;
        let mut rng = thread_rng();
        let mut i = 0usize;
        while i < len {
            let r = rng.gen_range(0..self.length) as usize;
            let mut end = i+r as usize;
            if end > len { end = len; }
            let mut span = pixels[i..=end].to_vec();
            spans.push(span);
            i += r + 1;
        }
        spans
    }
}
