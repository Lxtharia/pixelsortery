use std::f64::consts::PI;

use image::{Rgb, RgbImage};

#[derive(Debug, Clone, Copy)]
pub enum PathCreator {
    AllHorizontally,
    AllVertically,
    HorizontalLines,
    VerticalLines,
    Circles,
    SquareSpiral,
    RectSpiral,
    Diagonally(f32),
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
        let mut all_paths = match self {
            PathCreator::AllHorizontally => path_all_horizontally(all_pixels, w, h),
            PathCreator::AllVertically => path_all_vertically(all_pixels, w, h),
            PathCreator::HorizontalLines => path_horizontal_lines(all_pixels, w, h),
            PathCreator::VerticalLines => path_vertical_lines(all_pixels, w, h),
            PathCreator::SquareSpiral => path_rect_spiral(all_pixels, w, h, true),
            PathCreator::RectSpiral => path_rect_spiral(all_pixels, w, h, false),
            PathCreator::Diagonally(angle) => path_diagonal_lines(all_pixels, w, h, angle),
            PathCreator::Circles => path_ellipses(all_pixels, w, h, true),
        };

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

fn path_diagonal_lines(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64, angle: f32) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for x in 0..w {
        let mut path = Vec::new();
        for y in 0..h {
            let xf = ( y as f32 * angle.to_radians().tan() ).round() as i32;
            let i = (y * w + x) as i32 + xf;
            path.push(i as u64);
        }
        paths.push(path);
    }

    return pick_pixels(all_pixels, paths);
}

fn path_rect_spiral(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64, square: bool) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
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
    let mut reach_x = 1;
    let mut reach_y = 1;
    if !square {
        if w>h {reach_x = std::cmp::max(1, w-h);}
        else {reach_y = std::cmp::max(1, h-w);}
        x -= reach_x/2;
        y -= reach_y/2;
    }

    loop {
        for _ in 0..reach_x {
            x += 1;
            add_pixel_at(x, y);
        }
        for _ in 0..reach_y {
            y += 1;
            add_pixel_at(x, y);
        }
        reach_x += 1;
        reach_y += 1;
        if reach_y >= max_size || reach_x >= max_size {
            break;
        };
        for _ in 0..reach_x {
            x -= 1;
            add_pixel_at(x, y);
        }
        for _ in 0..reach_y {
            y -= 1;
            add_pixel_at(x, y);
        }
        reach_x += 1;
        reach_y += 1;
        if reach_y >= max_size || reach_x >= max_size {
            break;
        };
    }
    paths.push(path);

    return pick_pixels(all_pixels, paths);
}

fn path_ellipses(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64, circle: bool) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut x = w as f64 / 2.0;
    let mut y = h as f64 / 2.0;
    let pixelcount = w*h;
    // The max radius has to be
    let max_size = ((w*w + h*h) as f64).sqrt().ceil() as u64;

    let mut r = 1;
    let angle_offset = -0.5 * PI;

    // TODO: let angle_offset be set
    // TODO: make elliptic, not just circles
    // TODO: Allow to set center

    while r <= max_size / 2 {
        let mut path = Vec::new();
        let step_amounts = 8 * r as u64;
        let circ_step_size: f64 = 2.0*PI / step_amounts as f64;
        for step in 0..=step_amounts {
            let angle = angle_offset + circ_step_size * step as f64;
            let xi = x + angle.cos() * r as f64;
            let yi = y + angle.sin() * r as f64;
            path.push(yi as u64 * w + xi as u64)
        }
        paths.push(path);
        r += 1;
    }

    return pick_pixels(all_pixels, paths)
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
