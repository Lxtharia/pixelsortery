use ::array_init::array_init;
use image::{RgbImage, Rgb, Pixel};
use crate::sorters::SortingMethod;
use crate::color_helpers::*;

pub fn mut_map_sort(pixels: &mut [&mut Rgb<u8>], method: &SortingMethod) {
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



pub fn mapsort(img:&RgbImage, width: u32, height: u32, method: &SortingMethod) -> RgbImage{
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


