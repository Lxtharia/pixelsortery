use crate::color_helpers::*;
use image::Rgb;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelDrainRange, ParallelIterator
};
use std::{cmp::min, collections::VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdSelector {
    pub min: u64,
    pub max: u64,
    pub criteria: PixelSelectCriteria,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomSelector {
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedSelector {
    pub len: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FullSelector {}

/// Key criteria which a (Threshold-)Selector should use as a key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelSelectCriteria {
    Hue,
    Brightness,
    Saturation,
}

/// Returns a list of pixel spans
pub trait PixelSelector {
    fn create_spans<'a>(
        &'a self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>>;
    fn info_string<'a>(&'a self) -> String; // I bet this is no the rust way
}

impl PixelSelector for FullSelector {
    fn info_string(&self) -> String {
        format!("Selecting all pixels")
    }
    fn create_spans<'a>(
        &'a self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

        let mut span: Vec<&mut Rgb<u8>> = Vec::new();
        while !pixels.is_empty() {
            span.push(pixels.pop_front().unwrap());
        }
        spans.push(span);
        spans
    }
}

impl PixelSelector for FixedSelector {
    fn info_string(&self) -> String {
        format!("Selecting ranges of fixed length {}", self.len)
    }
    fn create_spans<'a>(
        &'a self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>> {

        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

        // Prevent an endless loop
        if self.len == 0 {
            return spans;
        }

        while pixels.len() >= self.len as usize {
            // Take len pixels and put into new span
            spans.push(pixels.drain(0..self.len as usize).collect());
        }
        // Push the rest
        spans.push(pixels.drain(..).collect());

        spans
    }
}

impl PixelSelector for RandomSelector {
    fn info_string(&self) -> String {
        format!("Random Selector with max length {}", self.max)
    }
    fn create_spans<'a>(
        &'a self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();
        // rng_range cannot be 1..1
        if self.max <= 1 {
            return spans;
        }
        let mut rng = thread_rng();
        let rng_range = Uniform::from(1..self.max as usize);

        loop {
            let mut r = rng_range.sample(&mut rng);
            if pixels.len() < r {break;}
            // Take r pixels and put into new span
            spans.push(pixels.drain(0..r).collect());
        }
        // Push the rest
        spans.push(pixels.drain(..).collect());

        spans
    }
}

impl PixelSelector for ThresholdSelector {
    fn info_string(&self) -> String {
        format!(
            "Selecting Pixels with: [{} < {:?} < {}]",
            self.min, self.criteria, self.max
        )
    }
    fn create_spans<'a>(
        &'a self,
        pixels: &mut VecDeque<&'a mut Rgb<u8>>,
    ) -> Vec<Vec<&'a mut Rgb<u8>>> {
        let mut spans: Vec<Vec<&'a mut Rgb<u8>>> = Vec::new();

        let value_function = match self.criteria {
            PixelSelectCriteria::Hue => get_hue,
            PixelSelectCriteria::Brightness => get_brightness,
            PixelSelectCriteria::Saturation => get_saturation,
        };

        // Function that checks if a value is valid
        let valid = |val| { (val as u64) >= self.min && (val as u64) <= self.max };

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
}
