use image::RgbImage;
use log::{error, info};
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{
        PixelSelectCriteria, PixelSelector
    },
    span_sorter::{SortingAlgorithm, SortingCriteria},
};
use std::{io::Read, path::PathBuf, str::FromStr};
use std::time::Instant;
use std::{collections::VecDeque, env, process::exit};

mod gui;

pub(crate) mod layers;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");


fn parse_parameter<T: FromStr>(arg: Option<String>, usage: &str) -> T {
    if let Some(s) = arg {
        if let Ok(n) = s.parse::<T>() {
            return n;
        }
    }
    eprintln!("[ERROR] Wrong syntax, usage {}", usage);
    exit(-1);
}

fn parse_thres_selector_parameters(arg: Option<String>) -> PixelSelector {
    // parse the string after that: --thres hue:10:200
    if let Some(arg2) = arg {
        let mut thres_opts: VecDeque<&str> = VecDeque::from_iter(arg2.split(":"));
        let (criteria, defaultmin, defaultmax) = match thres_opts.pop_front().unwrap_or("") {
            "hue" => (PixelSelectCriteria::Hue, 0, 360),
            "bright" => (PixelSelectCriteria::Brightness, 0, 255),
            "sat" => (PixelSelectCriteria::Saturation, 0, 255),
            _ => {
                eprintln!("[ERROR] Wrong syntax, usage: --thres <hue|bright|sat>:0:255");
                exit(-1)
            }
        };
        let min = thres_opts
            .pop_front()
            .unwrap_or("")
            .parse()
            .unwrap_or(defaultmin);
        let max = thres_opts
            .pop_front()
            .unwrap_or("")
            .parse()
            .unwrap_or(defaultmax);
        return PixelSelector::Threshold { min, max, criteria,
        };
    } else {
        eprintln!("[ERROR] Wrong syntax, usage: --thres <hue|bright|sat>:0:255");
        exit(-1)
    }
}

const HELP_STRING: &str = "
=================== Pixelsorter ===================
   Usage: pixelsortery <infile> --output|-o <outfile> [<options>]
   If <infile>  is '-' , read from stdin
   If <outfile> is '-' , write to stdout

===================== Options ====================

   -h | --help    : Show this and exit
   -v | --version : Show version and exit
   --quiet        : Make the program shut up
   --show-mask    : Outputs a mask showing what areas would be sorted (needs --thres)

================ Direction Options ==============

   --vertical   : Sort all pixels top to bottom, left to right
   --horizontal : Sort all pixels left to right, top to bottom
   --right      : Sort horizontal lines of pixels to the right
   --left       : Sort horizontal lines of pixels to the left
   --down       : Sort vertical lines of pixels downwards
   --up         : Sort vertical lines of pixels upwards
   --circles          : Sort in circles
   --spiral-square    : Sort in a squared spiral
   --spiral-rect      : Sort in a rectangular spiral
   --diagonal <angle> : Sort lines tilted by an angle
   --reverse          : Sort in the opposite direction

============= Span-Selection  Options ===========
  [Choose which pixels are valid to form a span]

   --random <max>                        : Sort spans of random length between 0 and <max>
   --fixed  <max>                        : Sort spans of a fixed length <max>
   --thres <hue|bright|sat>:<min>:<max>  : Mark pixels as valid if [hue|bright|sat] is between <min> and <max>

================= Sorting Options ===============

   --hue        : Sort Pixels by Hue
   --saturation : Sort Pixels by Saturation
   --brightness : Sort Pixels by Brightness

================ Algorithm Options ==============

   --mapsort    : Default. O(n)
   --shellsort  : Also cool.
   --glitchsort : Used to create a glitch-like effect
";


fn main() {

    let mut args: VecDeque<String> = env::args().collect();
    // shift
    args.pop_front();

    // ENABLE LOGGING WITH A LOGGING LEVEL
    let mut log_builder = env_logger::builder();
    log_builder.format_timestamp(None);
    log_builder.format_target(false);
    // Disable logging when --quiet is given
    if args.contains(&String::from("--quiet")){
        log_builder.filter_level(log::LevelFilter::Off);
    }
    log_builder.init();

    // Try to match first argument as path
    let path;
    if let Some(s) = args.pop_front() {
        match s.as_str() {
            "--help" | "-h" | "" => { println!("{}", HELP_STRING); exit(0); }
            "--version" | "-V" => { println!("{} {}", PACKAGE_NAME, VERSION); exit(0); }
            _ => path = s,
        };
    } else {
        eprintln!("No arguments passed. Starting gui...");
        gui::start_gui().unwrap();
        exit(0);
    }

    // OPEN IMAGE OR READ FROM STDIN
    let mut img: RgbImage = match path.as_str(){
        "-" => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();
            image::load_from_memory(&buf).unwrap().into_rgb8()
        },
        _ => image::open(&path).unwrap().into_rgb8(),
    };


    // CREATE DEFAULT PIXELSORTER
    let mut ps = pixelsortery::Pixelsorter::new();
    let mut do_reverse = false;

    let mut output_path = String::new();
    let mut show_mask = false;

    let mut start_gui = false;

    // I should just use some argument library tbh
    while let Some(arg) = args.pop_front() {
        match arg.as_str() {
            "-h" | "--help"    => { println!("{}", HELP_STRING); exit(0); }
            "-V" | "--version" => { println!("{} {}", PACKAGE_NAME, VERSION); exit(0); }
            "-o" | "--output"  => { output_path = parse_parameter::<String>(args.pop_front(), "--output") }

            "--random" => ps.selector = PixelSelector::Random { max: parse_parameter(args.pop_front(), "--random <max>")},
            "--fixed"  => ps.selector = PixelSelector::Fixed  { len: parse_parameter(args.pop_front(), "--fixed <len>")},
            "--thres"  => ps.selector = parse_thres_selector_parameters(args.pop_front()),

            "--vertical"   => ps.path_creator = PathCreator::AllVertically,
            "--horizontal" => ps.path_creator = PathCreator::AllHorizontally,
            "--right"      => ps.path_creator = PathCreator::HorizontalLines,
            "--left"       => { ps.path_creator = PathCreator::HorizontalLines; ps.reverse = true},
            "--down"       =>   ps.path_creator = PathCreator::VerticalLines,
            "--up"         => { ps.path_creator = PathCreator::VerticalLines;   ps.reverse = true},
            "--circles"           =>   ps.path_creator = PathCreator::Circles,
            "--spiral"            =>   ps.path_creator = PathCreator::Spiral,
            "--spiral-square"     =>   ps.path_creator = PathCreator::SquareSpiral,
            "--spiral-rect"       =>   ps.path_creator = PathCreator::RectSpiral,
            "--diagonal"   => ps.path_creator = PathCreator::Diagonally(parse_parameter(args.pop_front(), "--diagonal <angle>")),
            "--hilbert"    =>   ps.path_creator = PathCreator::Hilbert,
            "--reverse"    => do_reverse = true,

            "--hue"         => ps.sorter.criteria = SortingCriteria::Hue,
            "--brightness"  => ps.sorter.criteria = SortingCriteria::Brightness,
            "--saturation"  => ps.sorter.criteria = SortingCriteria::Saturation,

            "--debugcolors" => ps.sorter.algorithm = SortingAlgorithm::DebugColor,
            "--glitchsort"  => ps.sorter.algorithm = SortingAlgorithm::Glitchsort,
            "--shellsort"   => ps.sorter.algorithm = SortingAlgorithm::Shellsort,
            "--mapsort"     => ps.sorter.algorithm = SortingAlgorithm::Mapsort,

            "--show-mask" => show_mask = true,

            "--gui" => start_gui = true,

            _ => {
                eprintln!("Unrecognized argument: {}", arg);
                exit(-1)
            }
        }
    }
    if do_reverse {
        ps.reverse = ! ps.reverse;
    }

    // Start gui with set options
    if start_gui {
        gui::start_gui_with_sorter(&ps, img, PathBuf::from(&path)).unwrap();
        exit(0);
    }

    // If no gui is opened, we need an output path
    if output_path.is_empty() {
        eprintln!("You need to specify the output! Usage: --output <FILE> | -o <FILE>");
        exit(-1)
    }

    // LET'S GO SORT
    let start = Instant::now();

    if show_mask {
        // Drawing a mask
        if let Err(_) = ps.mask(&mut img){
            error!("Couldn't create mask. Masking is only possible with the threshold selector.");
            exit(-1);
        }
    } else {
        // SORTING
        ps.sort(&mut img);
    }

    let duration = start.elapsed();
    info!("=> TIME [Total]:\t{:?}\n", duration);

    // SAVING
    match output_path.as_str() {
        "-" => {
            info!("Saving to stdout");
            let _ = img.write_with_encoder(image::codecs::png::PngEncoder::new(std::io::stdout()));
        },
        _ => {
            info!("Saving to {}", output_path);
            let _ = img.save(&output_path);
        }
    }
}


