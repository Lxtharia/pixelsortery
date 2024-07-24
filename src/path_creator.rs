use image::{Rgb, RgbImage};

#[derive(Debug, Clone, Copy)]
pub enum PathCreator {
    AllHorizontally,
    AllVertically,
    HorizontalLines,
    VerticalLines,
    SquareSpiral,
}

impl PathCreator {
    pub fn info_string(self) -> String {
        format!("Direction/Order: [{:?}]", self)
    }
    pub fn create_paths(self, img: &mut RgbImage, reverse: bool) -> Vec<Vec<&mut Rgb<u8>>> {
        let w: u64 = img.width().into();
        let h: u64 = img.height().into();

        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();

        // Ideas/missing:
        // Hilbert Curve
        // Diagonally
        // In a Spiral
        // In circles
        let pathing_function = match self {
            PathCreator::AllHorizontally => path_all_horizontally,
            PathCreator::AllVertically => path_all_vertically,
            PathCreator::HorizontalLines => path_horizontal_lines,
            PathCreator::VerticalLines => path_vertical_lines,
            PathCreator::SquareSpiral => path_square_spiral,
        };
        let mut all_paths = pathing_function(all_pixels, w, h);
        if reverse {
            all_paths.iter_mut().for_each(|p| {
                p.reverse();
            });
        }
        return all_paths;
    }
}

fn path_all_horizontally(all_pixels: Vec<&mut Rgb<u8>>, _: u64, _: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    vec![all_pixels]
}

fn path_all_vertically(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut path = Vec::new();
    for x in 0..w {
        for y in 0..h {
            let i = y * w + x;
            path.push(i);
        }
    }
    paths.push(path);
    return pick_pixels(all_pixels, paths);
}

fn path_horizontal_lines(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for y in 0..h {
        paths.push((y * w..y * w + w).collect());
    }

    return pick_pixels(all_pixels, paths);
}

fn path_vertical_lines(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for x in 0..w {
        let mut path = Vec::new();
        for y in 0..h {
            let i = y * w + x;
            path.push(i);
        }
        paths.push(path);
    }

    return pick_pixels(all_pixels, paths);
}

fn path_square_spiral(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    // Ints will get floored
    let mut x = w / 2;
    let mut y = h / 2;
    let pixelcount = w*h;
    let max_size = std::cmp::max(w, h);

    let mut path = Vec::new();
    let mut add_pixel_at = |x: u64, y: u64| {
        let i = y * w + x;
        if i < pixelcount {
            path.push(i)
        }
    };

    // Add pixel in the middle
    add_pixel_at(x, y);
    let mut reach = 1;
    loop {
        for _ in 0..reach {
            x += 1;
            add_pixel_at(x, y);
        }
        for _ in 0..reach {
            y += 1;
            add_pixel_at(x, y);
        }
        reach += 1;
        if reach >= max_size {
            break;
        };
        for _ in 0..reach {
            x -= 1;
            add_pixel_at(x, y);
        }
        for _ in 0..reach {
            y -= 1;
            add_pixel_at(x, y);
        }
        reach += 1;
        if reach >= max_size {
            break;
        };
    }
    paths.push(path);

    return pick_pixels(all_pixels, paths);
}

/// Creates and returns ranges of mutable Pixels.
/// The picked pixels and their order are determined by the given vector of indices
fn pick_pixels(all_pixels: Vec<&mut Rgb<u8>>, indices: Vec<Vec<u64>>) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<&mut Rgb<u8>>> = Vec::new();

    let mut all_pixels: Vec<Option<&mut Rgb<u8>>> =
        all_pixels.into_iter().map(|p| Some(p)).collect();
    for li in indices {
        let mut path = Vec::new();
        for i in li {
            all_pixels.push(None);
            // Check if the index is valid
            if all_pixels.get(i as usize).is_some() {
                // Check if the pixel at index i is still available (not None)
                if let Some(px) = all_pixels.swap_remove(i as usize) {
                    path.push(px);
                }
            }
        }
        paths.push(path);
    }

    return paths;
}
