use image::{RgbImage, Rgb, Pixel};
use crate::Span;

pub trait PixelSelector {
    /// Returns a list of pixel spans
    fn spans(&self, pixels: &Vec<&mut Rgb<u8>>) -> Vec<Span>;
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
    fn spans(&self, pixels: &Vec<&mut Rgb<u8>>) -> Vec<Span> {
        unimplemented!()
    }
}
