#![allow(unused)]

use std::{collections::VecDeque, env, process::exit};
use image::RgbImage;
use std::time::{Duration, Instant};
use pixelsorter::sorter::{Sorter, SortingMethod};

fn main() {
    let mut args: VecDeque<String> = env::args().collect();
    // shift
    args.pop_front();

    let path =  match args.get(0) {
        Some(s) => s,
        None => {println!("[!] You need to specify the input and the output path"); exit(1);},
    };

    let output_path =  match args.get(1) {
        Some(s) => s,
        None => {println!("[!] You need to specify the output path"); exit(1);},
    };
    
    // let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    
    // OPEN IMAGE 
    let img: RgbImage = image::open(path).unwrap().into_rgb8();

    // SORTING 
    println!("Sorting all the pixels...");
    let start = Instant::now();

    // sort_whole_image(&mut img, &SortingMethod::Saturation);
    // sort_whole_image(&mut img, &SortingMethod::Brightness);
    // sort_whole_image(&mut img, &SortingMethod::Hue);
    let sorter = Sorter {method: SortingMethod::Saturation};
    let mut ps = pixelsorter::Pixelsorter::new(img, sorter);
    ps.sort();
    ps.sorter.method = SortingMethod::Hue;
    ps.sort();
    ps.sorter.method = SortingMethod::Debug;
    ps.sort();

    let duration = start.elapsed();
    println!("Time took to sort: {:?}", duration);

    /* SAVING */
    // let serial_num = 6;
    // let filename_mut = format!("./renatus-b-debug-{}.png", serial_num);
    // let filename_out = format!("./outtest-{}.png", serial_num);
    let _ = ps.save(output_path);
    // sorted_img_hb.save(filename_out);
}


