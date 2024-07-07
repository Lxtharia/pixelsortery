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

    let mut path = String::from("");
    if let Some(s) = args.pop_front() {
        match s.as_str() {
            "--help" | "-h" | "" => {
                println!("
    =========== Pixelsorter ===========
     Usage: pixelsort <infile> <outfile> [<options>]
    ============= Options =============
    --help | -h : Show this
    ===== Sorting Options
    --hue        : Sort Pixels by Hue
    --saturation : Sort Pixels by Saturation
    --brightness : Sort Pixels by Brightness
    ===== Span-Selection options. Choose which pixels are valid to form a span
    --thres [hue|bright|sat]:<min>:<max>  : Mark pixels as valid if [hue|bright|sat] is between <min> and <max>
                ");
                exit(0)
            },
            _ => path = s,
        };
    } else {
        println!("[!] You need to specify the input and the output path");
        exit(1);
    }
    let output_path = match args.pop_front() {
        Some(s) => s,
        None => {
            println!("[!] You need to specify the output path");
            exit(1);
        }
    };

    // OPEN IMAGE
    let img: RgbImage = image::open(path).unwrap().into_rgb8();
    // CREATE PIXELSORTER
    let mut ps = pixelsorter::Pixelsorter::new(img);

    // I should just use some argument library tbh
    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "--hue" => ps.sorter.criteria = SortingCriteria::Hue,
            "--brightness" => ps.sorter.criteria = SortingCriteria::Brightness,
            "--saturation" => ps.sorter.criteria = SortingCriteria::Saturation,
            "--debugcolors" => ps.sorter.criteria = SortingCriteria::Debug,
            "--thres" => {
                let mut thres = ThresholdSelector {
                    min: 0,
                    max: 360,
                    criteria: PixelSelectCriteria::Hue,
                };
                // parse the string after that: --thres hue:10:200
                if let Some(arg2) = args.pop_front() {
                    let mut thres_opts: VecDeque<&str> = VecDeque::from_iter(arg2.split(":"));
                    let (crit, defaultmin, defaultmax) =
                        match thres_opts.pop_front().unwrap_or("hue") {
                            "hue" => (PixelSelectCriteria::Hue, 0, 360),
                            "bright" => (PixelSelectCriteria::Brightness, 0, 100),
                            "sat" => (PixelSelectCriteria::Saturation, 0, 100),
                            _ => (PixelSelectCriteria::Hue, 0, 360),
                        };
                    thres.criteria = crit;
                    thres.min = thres_opts
                        .pop_front()
                        .unwrap_or("")
                        .parse()
                        .unwrap_or(defaultmin);
                    thres.max = thres_opts
                        .pop_front()
                        .unwrap_or("")
                        .parse()
                        .unwrap_or(defaultmax);
                } else {
                    println!("[WARNING!][Flag Usage:] --thresh [hue|bright|sat]:<min>:<max> ");
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
