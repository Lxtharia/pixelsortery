#![allow(unused_parens)]
use image::{Pixel, Rgb};
use std::cmp::{max, min};

/// Returns the hue value of a pixel in the range [0, 360]
pub fn get_hue(&pixel: &Rgb<u8>) -> u16 {
    let channels = pixel.channels();
    let r: f32 = channels[0] as f32 / 255.0;
    let g: f32 = channels[1] as f32 / 255.0;
    let b: f32 = channels[2] as f32 / 255.0;
    let mut hue: f32 = 0.0;

    if (r >= g && g >= b) {
        //orange
        if r == b {
            return 0;
        }
        hue = 60.0 * (g - b) / (r - b);
    } else if (g > r && r >= b) {
        //Chartreuse
        hue = 120.0 - 60.0 * (r - b) / (g - b);
    } else if (g >= b && b > r) {
        //green
        hue = 120.0 + 60.0 * (b - r) / (g - r);
    } else if (b > g && g > r) {
        //azure
        hue = 240.0 - 60.0 * (g - r) / (b - r);
    } else if (b > r && r >= g) {
        //violet
        hue = 240.0 + 60.0 * (r - g) / (b - g);
    } else if (r >= b && b > g) {
        //rose
        hue = 360.0 - 60.0 * (b - g) / (r - g);
    }

    return hue as u16;
}

/// returns the brightness of a pixel in the range [0, 255]
pub fn get_brightness(&p: &Rgb<u8>) -> u16 {
    let channels = p.channels();
    (0.2126 * channels[0] as f32 + 0.7152 * channels[1] as f32 + 0.0722 * channels[2] as f32) as u16
}

/// returns the saturation of a pixel in the range [0, 255]
pub fn get_saturation(&p: &Rgb<u8>) -> u16 {
    let channels = &p.channels();
    let maxrgb: u16 = max(channels[0], max(channels[1], channels[2])).into();
    if maxrgb == 0 {
        return 0_u16;
    }
    let minrgb: u16 = min(channels[0], min(channels[1], channels[2])).into();
    (255 * (maxrgb - minrgb) / maxrgb) as u16
}
