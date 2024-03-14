#![allow(unused)]

use image::RgbImage;
use std::time::{Duration, Instant};
use pixelsorter::sorters::{SortingMethod};

fn main() {
    let path: &str = "/home/xlein/Pictures/Wallpaper/landscape renatus.z wallpaper.jpg";
//    let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    
    // OPEN IMAGE 
    let img: RgbImage = image::open(path).unwrap().into_rgb8();

    // SORTING 
    println!("Sorting all the pixels...");
    let start = Instant::now();

//    sort_whole_image(&mut img, &SortingMethod::Saturation);
//    sort_whole_image(&mut img, &SortingMethod::Brightness);
//    sort_whole_image(&mut img, &SortingMethod::Hue);
    let mut ps = pixelsorter::Pixelsorter::new(img, SortingMethod::Saturation);
    ps.sort();
    ps.method = SortingMethod::Hue;
    ps.sort();
    ps.method = SortingMethod::Debug;
    ps.sort();

    let duration = start.elapsed();
    println!("Time took to sort: {:?}", duration);
    
    /* SAVING */
    let serial_num = 6;
    let filename_mut = format!("./renatus-b-debug-{}.png", serial_num);
    // let filename_out = format!("./outtest-{}.png", serial_num);
    let _ = ps.save(filename_mut);
    // sorted_img_hb.save(filename_out);
}


