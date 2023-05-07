#![allow(unused)]
use ::array_init::array_init;
use image::{RgbImage, Rgb, Pixel};

fn main() {
    let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    let img: RgbImage = image::open(path).unwrap().into_rgb8();
    

    let sorted_img: RgbImage = mapsort(&img, img.width(), img.height());
    println!("saving...");
    sorted_img.save(format!("./out-{}.png",2));
}

fn get_hue(&pixel: &Rgb<u8>) -> usize {
    let channels = pixel.channels();
    let r: f32 =  channels[0] as f32 / 255.0;
    let g: f32 =  channels[1] as f32 / 255.0;
    let b: f32 =  channels[2] as f32 / 255.0;
    let mut hue: f32 = 0.0;

    if (r >= g && g >= b){
        //orange
        if r == b {return 0;}
        hue = 60.0 * (g-b)/(r-b);
    }
    else if (g > r && r >= b){
        //Chartreuse
        hue = 120.0 -  60.0 * (r-b)/(g-b);
    }
    else if (g >= b && b > r){
        //green
        hue = 120.0 + 60.0 * (b-r)/(g-r);
    }
    else if (b > g && g > r){
        //azure
        hue = 240.0 - 60.0 * (g-r)/(b-r);
    }
    else if (b > r && r >= g){
        //violet
        hue = 240.0 + 60.0 * (r-g)/(b-g);
    }
    else if (r >= b && b > g){
        //rose
        hue = 360.0 - 60.0 * (b-g)/(r-g);
    }

    return hue as usize;
}

fn get_brightness(&p: &Rgb<u8>) -> usize {
    let channels = p.channels();
    (0.2126*channels[0] as f32 + 0.7152*channels[1] as f32 + 0.0722*channels[2] as f32) as usize
}

fn mapsort(img:&RgbImage, width: u32, height: u32) -> RgbImage{
    let pixels = img.pixels();
    let mut sorted: RgbImage = RgbImage::new(width, height);
    let mut hue_map: [ Vec<&Rgb<u8>> ; 360] = array_init(|_| Vec::new());

    println!("Mapping hue...");
    for p in pixels{
       //println!("{:?}: {}\t", &p, get_hue(&p));
        hue_map[get_brightness(&p) as usize].push(&p);
    }
    
    println!("Writing pixels...");
    let mut ind = 0;
    for h in hue_map {
        for p in h {
            sorted.put_pixel(ind%width, ind/width, *p);
            ind += 1;
        }
    }

    println!("Done!");
    return sorted;
}


