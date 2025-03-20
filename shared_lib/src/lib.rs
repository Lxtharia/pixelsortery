use pixelsortery::*;
use path_creator::PathCreator;
use pixel_selector::*;
use span_sorter::*;
use image::RgbImage;

/// This will be compiled into a shared library that provides the single function swaylock_effect
/// The function receives image data as an array of pixels, represented by an u32
///
/// Why: Experimenting with shared libraries and the FFI of Rust and C
/// and for using pixelsorting as a lockscreen effect in sway (see https://github.com/mortie/swaylock-effects/?tab=readme-ov-file#custom)

#[unsafe(no_mangle)]
pub extern "C" fn swaylock_effect(data: *mut u32, width: u32, height: u32) {

    let mut img = ppm_to_image(data, width, height);

    // This needs to be changed during compile time
    let sorters = hilbert_glitch();

    for sorter in sorters {
        sorter.sort(&mut img);
    }
    println!("[DONE] Sorting");
    image_to_ppm(data, &img);
    // Debug
    //let _ = img.save("/tmp/sorted.png");
}


fn ppm_to_image(data: *mut u32, width: u32, height: u32) -> RgbImage {
    unsafe {
        //println!("DATA: {:x}", *((data as u64 + 1) as *mut u32));
        let mut buf: Vec<u8> = Vec::with_capacity((width*height) as usize);

        (0..width*height).for_each(|i| {
            let i = i as u64;
            let ptr = data as u64 + 4*i;
            let px = *( ptr as *mut u32);
            // buf: r,g,b,a
            // data: a,r,g,b
            let r = (px >> 16) as u8;
            let g = (px >> 8) as u8;
            let b = (px >> 0) as u8;
            //let a = (px >> 24) as u8; // Weg mit a
            //if (i < 50) { println!("data: {:x} {},{},{}", px, r,g,b); }
            buf.push(r);
            buf.push(g);
            buf.push(b);
            //buf.push(a);

        });
        println!("[DONE] Converting int/pixels");

        //let img: RgbImage = image::RgbImage::from_vec(width, height, buf).expect("Buffer not big enough");
        image::RgbImage::from_vec(width, height, buf).expect("Buffer not big enough")
    }
}

fn image_to_ppm(data: *mut u32, img: &RgbImage) {
    let amount: u64 = ( img.width()*img.height()).into();
    let pxs = img.as_raw();
    unsafe{
        for i in 0..amount as usize {
            let r = (pxs[0 + 3*i as usize] as u32) << 16;
            let g = (pxs[1 + 3*i as usize] as u32) << 8;
            let b = (pxs[2 + 3*i as usize] as u32) << 0;
            //let a = (pxs[0 + 3*i as usize] as u32) << 24;
            *(((data as usize)+4*i) as *mut u32) = r | g | b;
        }
    }
}

//fn image_to_ppm(data: *mut u32, img: &RgbImage) {
//    let amount: u64 = (4 * img.width()*img.height()).into();
//    let pxs = img.as_raw();
//    unsafe{
//        for i in (0..amount).step_by(4) {
//            let r = (pxs[3 + i as usize] as u32) << 0;
//            let g = (pxs[2 + i as usize] as u32) << 8;
//            let b = (pxs[1 + i as usize] as u32) << 16;
//            let a = (pxs[0 + i as usize] as u32) << 24;
//            *(((data as u64)+i) as *mut u32) = r | g | b | a;
//        }
//    }
//}


fn hilbert_glitch() -> Vec<Pixelsorter> {
    let mut glitch = Pixelsorter::new();
    glitch.path_creator = PathCreator::HorizontalLines;
    glitch.sorter.algorithm = SortingAlgorithm::Glitchsort;

    let mut hilb = Pixelsorter::new();
    hilb.path_creator = PathCreator::Hilbert;
    hilb.selector = PixelSelector::Random { max: 500 };
    vec![hilb,glitch]
}

fn fixed_left() -> Vec<Pixelsorter> {
    let mut ps = Pixelsorter::new();
    ps.path_creator = PathCreator::HorizontalLines;
    ps.selector = PixelSelector::Fixed { len: 40 };
    vec![ps]
}

