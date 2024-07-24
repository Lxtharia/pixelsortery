use image::RgbImage;
use pixelsortery::path_creator::PathCreator;
use pixelsortery::pixel_selector::{
    FixedSelector, PixelSelectCriteria, PixelSelector, RandomSelector, ThresholdSelector
};
use pixelsortery::span_sorter::{SortingAlgorithm, SortingCriteria};
use std::time::Instant;
use std::{collections::VecDeque, env, process::exit};

fn parse_diagonal_path_parameters(arg: Option<String>) -> PathCreator {
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<f32>() {
            return PathCreator::Diagonally(n);
        }
    }
    println!("[ERROR] Wrong syntax, usage --diagonal <angle>");
    exit(-1);
}

fn parse_fixed_selector_parameters(arg: Option<String>) -> Box<dyn PixelSelector> {
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<u64>() {
            return Box::new(FixedSelector { len: n });
        }
    }
    println!("[ERROR] Wrong syntax, usage --fixed <max>");
    exit(-1);
}

fn parse_random_selector_parameters(arg: Option<String>) -> Box<dyn PixelSelector> {
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<u32>() {
            return Box::new(RandomSelector {max: n } );
        }
    }
    println!("[ERROR] Wrong syntax, usage --random <max>");
    exit(-1);
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

fn print_help() {
    println!("
    =========== Pixelsorter ===========
       Usage: pixelsort <infile> <outfile> [<options>]
    ============= Options =============
     --help | -h : Show this
    ===== Direction Options
     --vert
     --vertical   : Sort all pixels top to bottom, left to right
     --horizontal : Sort all pixels left to right, top to bottom
     --right      : Sort horizontal lines of pixels to the right
     --left       : Sort horizontal lines of pixels to the left
     --down       : Sort vertical lines of pixels downwards
     --up         : Sort vertical lines of pixels upwards
     --spiral-square : Sort in a squared spiral
     --spiral-rect   : Sort in a rectangular spiral
     --diagonal <angle> : Sort lines in an angle
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
}

fn main() {
    let mut args: VecDeque<String> = env::args().collect();
    // shift
    args.pop_front();

    let path;
    if let Some(s) = args.pop_front() {
        match s.as_str() {
            "--help" | "-h" | "" => { print_help(); exit(0); }
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
    let mut do_reverse = false;

    // I should just use some argument library tbh
    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "-h" | "--help" => { print_help(); exit(0); }
            "--random" => ps.selector = parse_random_selector_parameters(args.pop_front()),
            "--fixed"  => ps.selector = parse_fixed_selector_parameters(args.pop_front()),
            "--thres"  => ps.selector = parse_thres_selector_parameters(args.pop_front()),

            "--horizontal" => ps.path_creator = PathCreator::AllHorizontally,
            "--vertical"
                | "--vert" => ps.path_creator = PathCreator::AllVertically,
            "--right"      => ps.path_creator = PathCreator::HorizontalLines,
            "--left"       => { ps.path_creator = PathCreator::HorizontalLines; ps.reverse = true},
            "--down"       =>   ps.path_creator = PathCreator::VerticalLines,
            "--up"         => { ps.path_creator = PathCreator::VerticalLines;   ps.reverse = true},
            "--spiral-square"     =>   ps.path_creator = PathCreator::SquareSpiral,
            "--spiral-rect"       =>   ps.path_creator = PathCreator::RectSpiral,
            "--diagonal"   => ps.path_creator = parse_diagonal_path_parameters(args.pop_front()),
            "--reverse"    => do_reverse = true,

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
    if do_reverse {
        ps.reverse = ! ps.reverse;
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
