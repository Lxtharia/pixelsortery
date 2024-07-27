use std::{io::Read, path::PathBuf};
use clap::{Parser, ValueEnum};
use image::RgbImage;
use pixelsortery::{path_creator::*, pixel_selector::*, Pixelsorter};

#[derive(Parser)] 
#[command(version, about, long_about= None)]
struct Cli {
    quiet: bool,
    verbose: u8,

    #[arg(required=true)]
    input_path: String,
    #[arg(required=true)]
    output_path: String,

    // Either a list of all directions like --left --spiral --circles 90, 
    #[arg(value_parser = direction_parser)]
    direction: Option<Vec<(PathCreator, bool)>>,
    reverse: bool,
   


}

fn direction_parser(s: &str) -> Result<(PathCreator, bool), String> {
    let mut reverse = false;
    let dir: PathCreator = match s {
        "left" => {reverse = true; PathCreator::HorizontalLines},
        _ => PathCreator::HorizontalLines,
    };
    Ok((dir, reverse))
}
fn selector_parser(s: &str) -> Result<Box<dyn PixelSelector>, String> {
    let sel: Box<dyn PixelSelector> = match s {
        "thres" => Box::new(ThresholdSelector{criteria: PixelSelectCriteria::Hue, min:0, max: 360}),
        _ => Box::new(FullSelector{}),
    };
    Ok(sel)
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
    Circles,
    Square,
    Rect,
    Spiral,
}

pub fn parse_cli() -> Pixelsorter {
    let args = Cli::parse();
    // OPEN IMAGE OR READ FROM STDIN
    let img: RgbImage = match args.input_path.as_str() {
        "-" => {
            let mut buf = Vec::new();
            std::io::stdin().read_to_end(&mut buf).unwrap();
            image::load_from_memory(&buf).unwrap().into_rgb8()
        },
        _ => image::open(args.input_path).unwrap().into_rgb8(),
    };
    return Pixelsorter::new(img);

}
