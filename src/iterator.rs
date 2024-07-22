use std::collections::VecDeque;

use image::{Rgb, RgbImage};

#[derive(Debug, Clone, Copy)]
pub enum ImageIterator {
    All,
    Horizontal,
    Vertical,
}

impl ImageIterator {
    pub fn traverse(self, img: &mut RgbImage) -> Vec<Vec<&mut Rgb<u8>>> {
        let w: u64 = img.width().into();
        let h: u64 = img.height().into();
        let pixelcount= w * h;

        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();

        let traversing_function =  match self {
            ImageIterator::All => traverse_all,
            ImageIterator::Horizontal => traverse_horizontal,
            ImageIterator::Vertical => traverse_vertical,
        };
        traversing_function(all_pixels, w, h)
    }

}


fn traverse_all(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    vec![all_pixels]
}

fn traverse_horizontal(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
        let mut all_pixels: VecDeque<&mut Rgb<u8>> = all_pixels.into();
        let mut prespans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();

        let mut prespan = Vec::new();
        for i in 0..all_pixels.len() {
            let px = all_pixels.pop_front().unwrap();

            // When last pixel in the line
            if i as u64 % w < w-1 {
                // A valid pixel. Add to span
                prespan.push(px);
            } else {
                // Add last pixel, push span and create a new one
                prespan.push(px);
                prespans.push(prespan);
                prespan = Vec::new();
            }
        }
        prespans.push(prespan);

        return prespans;
}

fn traverse_vertical(all_pixels: Vec<&mut Rgb<u8>>, w :u64, h :u64) -> Vec<Vec<&mut Rgb<u8>>> {
        let mut prespans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        todo!();
        return prespans;
}
