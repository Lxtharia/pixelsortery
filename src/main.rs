#![allow(unused)]
use ::array_init::array_init;
use image::{RgbImage, Rgb, Pixel};
use std::cmp::max;
use std::cmp::min;

#[derive(Debug)]
enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
}

fn main() {
    let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    let img: RgbImage = image::open(path).unwrap().into_rgb8();
 
    let sorted_img_h: RgbImage = mapsort(&img, img.width(), img.height(), SortingMethod::Hue);
    let filename_out = format!("./out-h-{}.png", 0);
    println!("saving as {}...", filename_out);
//    sorted_img_h.save(filename_out);

    let sorted_img_b: RgbImage = mapsort(&img, img.width(), img.height(), SortingMethod::Brightness);
    let filename_out = format!("./out-b-{}.png", 0);
    println!("saving as {}...", filename_out);
 //   sorted_img_b.save(filename_out);

    let sorted_img_hb: RgbImage = mapsort(&sorted_img_b, img.width(), img.height(), SortingMethod::Hue);
    let filename_out = format!("./out-hb-{}.png", 0);
    println!("saving as {}...", filename_out);
    sorted_img_hb.save(filename_out);

    let mut inv_img = img.clone();
    invert(&mut inv_img, img.width(), img.height());

    let filename_out = format!("./inverted-{}.png", 0);
    println!("saving as {}...", filename_out);
    inv_img.save(filename_out);


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

fn get_saturation(&p: &Rgb<u8>) -> usize{
    let channels = &p.channels();
    let maxrgb = max(channels[0], max(channels[1], channels[2]));
    if maxrgb == 0 {return 0_usize}
    let minrgb = min(channels[0], min(channels[1], channels[2]));
    (255*( maxrgb - minrgb ) / maxrgb) as usize
}


fn mapsort(img:&RgbImage, width: u32, height: u32, method: SortingMethod) -> RgbImage{
    let pixels = img.pixels();
    let mut sorted: RgbImage = RgbImage::new(width, height);
    let mut map_array: [ Vec<&Rgb<u8>> ; 360] = array_init(|_| Vec::new());
   
    use SortingMethod::*;
    let get_pixel_value = match method {
        Hue => get_hue,
        Brightness => get_brightness,
        Saturation => get_saturation,
    };
 
    println!("Mapping pixel value by {:?}...", method);
    for p in pixels{
       //println!("{:?}: {}\t", &p, get_hue(&p));
        map_array[get_pixel_value(&p)].push(&p);
    }
    
    println!("Writing pixels...");
    let mut ind = 0;
    for h in map_array {
        for p in h {
            sorted.put_pixel(ind%width, ind/width, *p);
            ind += 1;
        }
    }

    println!("Done!");
    return sorted;
}

fn invert (img: &mut RgbImage, width: u32, height: u32){
    let mut pixels = img.pixels_mut();
    
    for p in pixels {
        p.invert();
//       let channels = (*p).channels_mut();
//       let temp: u8 = channels[0];
//       channels[0] = channels[1];
//       channels[1] = channels[2];
//       channels[2] = temp;
    }
}
