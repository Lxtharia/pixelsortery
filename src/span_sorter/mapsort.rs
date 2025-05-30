use ::array_init::array_init;
use image::{Rgb, RgbImage};

/// Sorts a slice of Pixels by either Hue, Brightness or Saturation.
/// It uses an array with 360 elements to map the values.
/// Probably panics if a compare function returns a value above 360
pub fn mapsort_mut(pixels: &mut [&mut Rgb<u8>], get_pixel_value: for<'a> fn(&'a Rgb<u8>) -> u16) {

    let mut map_array: [Vec<Rgb<u8>>; 360] = array_init(|_| Vec::new());

    // we copy the pixels into the map array
    for p in 0..pixels.len() {
        map_array[get_pixel_value(&pixels[p]) as usize].push(pixels[p].clone());
    }

    // and then put them back at the pointer locations
    let mut ind = 0;
    for h in map_array {
        for p in h {
            *(pixels[ind]) = p;
            ind += 1;
        }
    }
}

/// Sorts all pixels of an image by either Hue, Brightness or Saturation.
/// It uses an array with 360 elements to map the values.
/// Probably panics if a compare function returns a value above 360
pub fn mapsort(img: &RgbImage, width: u32, height: u32, get_pixel_value: for<'a> fn(&'a Rgb<u8>) -> u16) -> RgbImage {
    let pixels = img.pixels();
    let mut sorted: RgbImage = RgbImage::new(width, height);
    let mut map_array: [Vec<&Rgb<u8>>; 360] = array_init(|_| Vec::new());

    for p in pixels {
        map_array[get_pixel_value(&p) as usize].push(&p);
    }

    let mut ind = 0;
    for h in map_array {
        for p in h {
            sorted.put_pixel(ind % width, ind / width, *p);
            ind += 1;
        }
    }

    return sorted;
}
