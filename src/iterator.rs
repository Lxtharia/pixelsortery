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
        let pixelcount = w * h;

        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();

        // Ideas/missing:
        // Hilbert Curve
        // Diagonally
        // In a Spiral
        let traversing_function = match self {
            ImageIterator::All => traverse_all,
            ImageIterator::Horizontal => traverse_horizontal,
            ImageIterator::Vertical => traverse_vertical,
        };
        traversing_function(all_pixels, w, h)
    }
    pub fn info_string(self) -> String {
        format!("Direction/Order: {:?}", self)
    }
}

fn traverse_all(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    vec![all_pixels]
}

fn traverse_horizontal(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut ranges: Vec<Vec<u64>> = Vec::new();

    for y in 0..h {
        ranges.push((y*w..y*w+w).collect());
    }

    return pick_pixels(all_pixels, ranges);
}

fn traverse_vertical(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut ranges: Vec<Vec<u64>> = Vec::new();

    for x in 0..w {
        let mut range = Vec::new();
        for y in 0..h {
            let i = (y * w + x);
            range.push(i);
        }
        ranges.push(range);
    }

    return pick_pixels(all_pixels, ranges);
}


/// Creates and returns ranges of mutable Pixels.
/// The picked pixels and their order are determined by the given indices vector
fn pick_pixels(all_pixels: Vec<&mut Rgb<u8>>, indices: Vec<Vec<u64>>) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut ranges: Vec<Vec<&mut Rgb<u8>>> = Vec::new();

    let mut all_pixels: Vec<Option<&mut Rgb<u8>>> =
        all_pixels.into_iter().map(|p| Some(p)).collect();
    for li in indices {
        let mut range = Vec::new();
        for i in li {
            all_pixels.push(None);
            if (all_pixels.get(i as usize).is_some()) {
                range.push(all_pixels.swap_remove(i as usize).unwrap());
            }
        }
        ranges.push(range);
    }

    return ranges;
}
