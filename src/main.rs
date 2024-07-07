#![allow(unused)]

use image::RgbImage;
use pixelsorter::pixel_selector::{PixelSelectCriteria, ThresholdSelector};
use pixelsorter::span_sorter::{SortingCriteria, SpanSorter};
use std::fmt::Arguments;
use std::time::{Duration, Instant};
use std::{collections::VecDeque, env, process::exit};

fn main() {
    let mut args: VecDeque<String> = env::args().collect();
    // shift
    args.pop_front();

    let path = match args.pop_front() {
        Some(s) => s,
        None => {
            println!("[!] You need to specify the input and the output path");
            exit(1);
        }
    };
    let output_path = match args.pop_front() {
        Some(s) => s,
        None => {
            println!("[!] You need to specify the output path");
            exit(1);
        }
    };

    // OPEN IMAGE
    let img: RgbImage = image::open(path).unwrap().into_rgb8();
    let mut ps = pixelsorter::Pixelsorter::new(img);

    // I should just use some argument library tbh
    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "--hue" => ps.sorter.criteria = SortingCriteria::Hue,
            "--thres" => {
                let mut thres = ThresholdSelector {
                    min: 100,
                    max: 200,
                    criteria: PixelSelectCriteria::Hue,
                };
                // parse the string after that: --thres hue:10:200
                if let Some(arg2) = args.pop_front() {
                    let mut thres_opts: Vec<&str> = arg2.split(":").collect();
                    thres.max = thres_opts.pop().unwrap_or("150").parse().unwrap_or(150);
                    thres.min = thres_opts.pop().unwrap_or("50").parse().unwrap_or(50);
                    thres.criteria = match thres_opts.pop().unwrap_or("hue") {
                        "hue" => PixelSelectCriteria::Hue,
                        _ => PixelSelectCriteria::Hue,
                    };
                }
                ps.selector = Box::new(thres);
            }
            _ => print!("Unrecognized argument: {}", arg),
        }
    }

    // SORTING
    println!("Sorting all the pixels...");
    let start = Instant::now();

    //    ps.sort();
    //    ps.sorter.criteria = SortingCriteria::Hue;
    //    ps.sort();
    //    ps.sorter.criteria = SortingCriteria::Debug;
    ps.sort();
    let duration = start.elapsed();

    /* SAVING */
    // let serial_num = 6;
    // let filename_mut = format!("./renatus-b-debug-{}.png", serial_num);
    // let filename_out = format!("./outtest-{}.png", serial_num);
    let _ = ps.save(&output_path);
    // sorted_img_hb.save(filename_out);
    //    ps.sort_all_pixels();
    ps.save(&output_path);
    ps.save(format!("full-{}", &output_path));

    println!("Time took to sort: {:?}", duration);
}
