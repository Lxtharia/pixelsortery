use image::{RgbImage, Rgb, Pixel};
use rand::{thread_rng, Rng};
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
        let spans: Vec<Span> = Vec::new();
        let len = pixels.len() as i32;
        let mut rng = thread_rng();
        let mut i = 0;
        while i < len {
            let r = rng.gen_range(0..self.length);
            let mut end = i+r;
            if end > len { end = len; }
            println!("Range from {} to {}", i, i+r);
            i += r + 1;
        }
       unimplemented!()
    }
}
