use image::Rgb;

#[derive(Debug)]
struct PixelWrapper{
    px:  Rgb<u8>,
    val: u16,
}

pub fn shellsort_mut(pixels: &mut [&mut Rgb<u8>], value_function: for<'a> fn(&'a Rgb<u8>) -> u16){
    // Stolen from some Stackoverflow Thread

    let span_len = pixels.len();
    let mut fake_pixels = Vec::new();
    // Wrap each pixel into a wrapper with a calculated value (TODO: from SortingCriteria)
    pixels.into_iter().for_each(|px| {
        let val = value_function(px);
        fake_pixels.push(PixelWrapper{px: **px, val});
    });

    let mut gap = span_len;
    let mut swapped = false;
    while ( (gap > 1) || swapped ) {
        if (gap > 1){ gap = (gap as f64/1.247330950103979) as usize; }
        swapped = false;
        for i in 0..span_len {
            if (gap + i >= span_len){break;}
            if ( fake_pixels[i+gap].val < fake_pixels[i].val ){
                fake_pixels.swap(i+gap, i);
                swapped = true;
            }
        }
    }

    for i in 0..pixels.len() {
        *pixels[i] = fake_pixels[i].px;
    }

}

