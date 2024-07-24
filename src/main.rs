use image::RgbImage;
use pixelsortery::path_creator::PathCreator;
use pixelsortery::pixel_selector::{
    FixedSelector, PixelSelectCriteria, PixelSelector, RandomSelector, ThresholdSelector
};
use pixelsortery::span_sorter::{SortingAlgorithm, SortingCriteria};
use std::time::Instant;
use std::{collections::VecDeque, env, process::exit};

fn parse_fixed_selector_parameters(arg: Option<String>) -> Box<dyn PixelSelector> {
    let mut fixed = FixedSelector { len: 200 };
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<u64>() {
            fixed.len = n;
        } else {
            println!("[ERROR] Wrong syntax, usage --fixed <max>");
            exit(-1);
        }
    } else {
        println!("[ERROR] Wrong syntax, usage: --fixed <max>");
        exit(-1);
    }
    Box::new(fixed)
}

fn parse_random_selector_parameters(arg: Option<String>) -> Box<dyn PixelSelector> {
    let mut rand = RandomSelector { max: 80 };
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<u32>() {
            rand.max = n;
        } else {
            println!("[ERROR] Wrong syntax, usage --random <max>");
            exit(-1);
        }
    } else {
        println!("[ERROR] Wrong syntax, usage: --random <max>");
        exit(-1);
    }
    Box::new(rand)
}

fn parse_thres_selector_parameters(arg: Option<String>) -> Box<dyn PixelSelector> {
    let mut thres = ThresholdSelector {
        min: 0,
        max: 360,
        criteria: PixelSelectCriteria::Hue,
    };
    // parse the string after that: --thres hue:10:200
    if let Some(arg2) = arg {
        let mut thres_opts: VecDeque<&str> = VecDeque::from_iter(arg2.split(":"));
        let (crit, defaultmin, defaultmax) = match thres_opts.pop_front().unwrap_or("") {
            "hue" => (PixelSelectCriteria::Hue, 0, 360),
            "bright" => (PixelSelectCriteria::Brightness, 0, 255),
            "sat" => (PixelSelectCriteria::Saturation, 0, 255),
            _ => {
                println!("[ERROR] Wrong syntax, usage: --thres <hue|bright|sat>:0:255");
                exit(-1)
            }
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
        println!("[ERROR] Wrong syntax, usage: --thres <hue|bright|sat>:0:255");
        exit(-1)
    }
    Box::new(thres)
}

fn main() {
    let mut args: VecDeque<String> = env::args().collect();
    // shift
    args.pop_front();

    let path;
    if let Some(s) = args.pop_front() {
        match s.as_str() {
            "--help" | "-h" | "" => {
                println!("
    =========== Pixelsorter ===========
       Usage: pixelsort <infile> <outfile> [<options>]
    ============= Options =============
     --help | -h : Show this
    ===== Direction Options
     --all        : Sort all pixels
     --vert
     --vertical   : Sort all columns of pixels
     --horizontal : Sort all lines of pixels
     --reverse    : Sort in the opposite direction
    ===== Sorting Options
     --hue        : Sort Pixels by Hue
     --saturation : Sort Pixels by Saturation
     --brightness : Sort Pixels by Brightness
    ===== Algorithm Options
     --mapsort    : Default. O(n)
     --shellsort  : Also cool.
     --glitchsort : Creates a glitch effect (Extremly cool)
    ===== Span-Selection options. Choose which pixels are valid to form a span
     --random <max>                        : Sort spans of random length between 0 and <max>
     --thres [hue|bright|sat]:<min>:<max>  : Mark pixels as valid if [hue|bright|sat] is between <min> and <max>
                ");
                exit(0)
            }
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
    // CREATE DEFAULT PIXELSORTER
    let mut ps = pixelsortery::Pixelsorter::new(img);

    // I should just use some argument library tbh
    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "--random" => ps.selector = parse_random_selector_parameters(args.pop_front()),
            "--fixed"  => ps.selector = parse_fixed_selector_parameters(args.pop_front()),
            "--thres"  => ps.selector = parse_thres_selector_parameters(args.pop_front()),

            "--all"        => ps.path_creator = PathCreator::All,
            "--horizontal" => ps.path_creator = PathCreator::Horizontal,
            "--vertical"
                | "--vert" => ps.path_creator = PathCreator::Vertical,
            "--reverse"    => ps.reverse = true,

            "--hue"         => ps.sorter.criteria = SortingCriteria::Hue,
            "--brightness"  => ps.sorter.criteria = SortingCriteria::Brightness,
            "--saturation"  => ps.sorter.criteria = SortingCriteria::Saturation,
            "--debugcolors" => ps.sorter.criteria = SortingCriteria::Debug,

            "--glitchsort" => ps.sorter.algorithm = SortingAlgorithm::Glitchsort,
            "--shellsort"  => ps.sorter.algorithm = SortingAlgorithm::Shellsort,
            "--mapsort"    => ps.sorter.algorithm = SortingAlgorithm::Mapsort,

            _ => {
                println!("Unrecognized argument: {}", arg);
                exit(-1)
            }
        }
    }
    
    let start = Instant::now();

    // SORTING
    ps.sort();

    let duration = start.elapsed();
    println!();
    println!("Total time: {:?}", duration);
    println!("Saving to {}", output_path);

    // SAVING
    let _ = ps.save(&output_path);
}
