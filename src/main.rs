#![allow(unused)]
mod color_helpers;
mod sorters;

use image::RgbImage;
use std::time::{Duration, Instant};
use sorters::{SortingMethod, sort_img, sort_whole_image};

fn main() {
    let path: &str = "/home/xlein/Pictures/Wallpaper/landscape renatus.z wallpaper.jpg";
//    let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    
    // OPEN IMAGE 
    let mut img: RgbImage = image::open(path).unwrap().into_rgb8();

    // SORTING 
    println!("Sorting all the pixels...");
    let start = Instant::now();

//    sort_whole_image(&mut img, &SortingMethod::Saturation);
//    sort_whole_image(&mut img, &SortingMethod::Brightness);
//    sort_whole_image(&mut img, &SortingMethod::Hue);
    sort_img(&mut img, &SortingMethod::Brightness);
    sort_img(&mut img, &SortingMethod::Hue);
    sort_img(&mut img, &SortingMethod::Debug);

    let duration = start.elapsed();
    println!("Time took to sort: {:?}", duration);
    
   // SAVING
    let serial_num = 6;
    let filename_mut = format!("./renatus-b-debug-{}.png", serial_num);
//    let filename_out = format!("./outtest-{}.png", serial_num);
    let _ = img.save(filename_mut);
//    sorted_img_hb.save(filename_out);
}


