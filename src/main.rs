#![allow(unused)]
use ::array_init::array_init;
use image::{RgbImage, Rgb, Pixel};
use std::cmp::max;
use std::cmp::min;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
}

fn main() {
    let path: &str = "/home/xlein/Pictures/Wallpaper/proxy.png";
    let mut img: RgbImage = image::open(path).unwrap().into_rgb8();

//    let mut inv_img = img.clone();
//    invert(&mut inv_img, img.width(), img.height());
//    let filename_out = format!("./inverted-{}.png", 0);
//    println!("saving as {}...", filename_out);
//    inv_img.save(filename_out);

let start = Instant::now();
    let sorted_img_b: RgbImage = mapsort(&img, img.width(), img.height(), &SortingMethod::Brightness);
    let sorted_img_hb: RgbImage = mapsort(&sorted_img_b, img.width(), img.height(), &SortingMethod::Hue);
let duration = start.elapsed();
println!("Time took to sort: {:?}", duration);

println!("Sorting all the pixels...");
let start = Instant::now();
//    sort_img(&mut img, &SortingMethod::Saturation);
    sort_whole_image(&mut img, &SortingMethod::Brightness);
    sort_whole_image(&mut img, &SortingMethod::Hue);
let duration = start.elapsed();
println!("Time took to sort: {:?}", duration);
    

println!("Sorting every two pixels...");
let start = Instant::now();
//    sort_img(&mut img, &SortingMethod::Saturation);
    sort_img(&mut img, &SortingMethod::Brightness);
    sort_img(&mut img, &SortingMethod::Hue);
let duration = start.elapsed();
println!("Time took to sort: {:?}", duration);
    
    let serial_num = 12;
    let filename_mut = format!("./muttest-{}.png", serial_num);
    let filename_out = format!("./outtest-{}.png", serial_num);
    img.save(filename_mut);
    sorted_img_hb.save(filename_out);

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

fn sort_whole_image(img: &mut RgbImage, method: &SortingMethod){
    let (width, height) = img.dimensions();
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    mut_map_sort(&mut pixels, method);
}

fn sort_img(img: &mut RgbImage, method: &SortingMethod){
    let (width, height) = img.dimensions();
    // a vector of pointers to the pixels
    let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
    // We are iterating through all lines.
    // What if we want to iterate through pixels diagonally?
    // Or in a hilbert curve?
    // So we need an array of iterators
    // each iterator needs to have mutable pixel pointers we can write to
    // Glitching is another construction site
    let mut k = 0;
    let mut start = 0;
    for y in 0..height {
        for x in 0..width {
            let index = y * width + x;

            if k>=2 || x >= width-1 {
                // we give mut_map_sort a mutable slice of RGB-pointers
                mut_map_sort(&mut pixels[start..start+k], method);
                start = 1+index as usize ;
                k = 0;
            } else {
                k+=1;
            }
        }
    }

}

fn mut_map_sort(pixels: &mut [&mut Rgb<u8>], method: &SortingMethod) {
    use SortingMethod::*;
    let get_pixel_value = match method {
        Hue => get_hue,
        Brightness => get_brightness,
        Saturation => get_saturation,
    };

    let mut map_array: [ Vec<Rgb<u8>> ; 360 ] = array_init(|_| Vec::new());
    
    // we copy the pixels into the map array 
    for p in 0..pixels.len(){
        map_array[get_pixel_value(&pixels[p])].push(pixels[p].clone());
        //map_array[get_pixel_value(&pixels[p])].push(Rgb {0: [255,0,0]});
    }

    // and then put them back at the pointer locations
    let mut ind = 0;
    for h in map_array {
        for p in h {
            *(pixels[ind]) = p; 
            ind+=1;
        }
    }

}

fn mapsort(img:&RgbImage, width: u32, height: u32, method: &SortingMethod) -> RgbImage{
    let pixels = img.pixels();
    let mut sorted: RgbImage = RgbImage::new(width, height);
    let mut map_array: [ Vec<&Rgb<u8>> ; 360] = array_init(|_| Vec::new());
   
    use SortingMethod::*;
    let get_pixel_value = match *method {
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
