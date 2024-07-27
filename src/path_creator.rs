use std::{f64::consts::PI, time::Instant};
use log::{info, warn, error};
use image::{Rgb, RgbImage};
use rayon::prelude::*;


#[derive(Debug, Clone, Copy)]
pub enum PathCreator {
    AllHorizontally,
    AllVertically,
    HorizontalLines,
    VerticalLines,
    Circles,
    Spiral,
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

        let mut timestart = Instant::now();
        timestart = Instant::now();
        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        info!("TIME | [Loading pixels]:\t+ {:?}", timestart.elapsed());
        timestart = Instant::now();
        // Ideas/missing:
        // Hilbert Curve
        // In waves
        let mut all_paths_indices = match self {
            PathCreator::AllHorizontally => path_all_horizontally(w, h),
            PathCreator::AllVertically => path_all_vertically(w, h),
            PathCreator::HorizontalLines => path_horizontal_lines(w, h),
            PathCreator::VerticalLines => path_vertical_lines(w, h),
            PathCreator::SquareSpiral => path_rect_spiral(w, h, true),
            PathCreator::RectSpiral => path_rect_spiral(w, h, false),
            PathCreator::Diagonally(angle) => path_diagonal_lines(w, h, angle),
            PathCreator::Circles => path_circles(w, h),
            PathCreator::Spiral => path_round_spiral(w, h),
        };
        info!("TIME | [Creating paths]:\t+ {:?}", timestart.elapsed());
        timestart = Instant::now();
        if reverse {
            all_paths_indices.iter_mut().for_each(|p| {
                p.reverse();
            });
        }
        info!("TIME | [Reversing paths]:\t+ {:?}", timestart.elapsed());
        return pick_pixels(all_pixels, all_paths_indices);
    }
}

fn path_all_horizontally(w: u64, h: u64) -> Vec<Vec<u64>> {
    vec![(0..w*h).collect()]
}

fn path_all_vertically(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut path = Vec::new();
    for x in 0..w {
        for y in 0..h {
            let i = y * w + x;
            path.push(i);
        }
    }
    paths.push(path);
    return paths;
}

fn path_horizontal_lines(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for y in 0..h {
        paths.push((y * w..y * w + w).collect());
    }

    return paths;
}

fn path_vertical_lines(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for x in 0..w {
        let mut path = Vec::new();
        for y in 0..h {
            let i = y * w + x;
            path.push(i);
        }
        paths.push(path);
    }

    return paths;
}

fn path_diagonal_lines(w: u64, h: u64, angle: f32) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    let tan_val = angle.to_radians().tan();
    // we need spans starting at x values outside our width, so that they "fly into" the image
    // For example: if spans go to the right at 45°, we need to start at -w/2, so that the bottom left pixel gets assigned a range as well
    let xoverhead = -(tan_val * h as f32).round() as i64;
    let xrange = if tan_val > 0.0 { xoverhead..w as i64 } else { 0..w as i64 + xoverhead };

    let line_path = |xs| {
        let mut path = Vec::new();
        for y in 0..h {
            let x = xs + ( y as f32 * tan_val ).round() as i64;
            // Prevent "overflowing" the index and selecting indices on the next line
            if x >= w as i64 || x < 0 { continue; }
            let i = y * w + x as u64;
            path.push(i);
        }
        path
    };
    // THREADPOOLING WOOO
    let path_iter = xrange.into_iter().map(line_path);
    paths = path_iter.collect();

    return paths;
}

fn path_rect_spiral(w: u64, h: u64, square: bool) -> Vec<Vec<u64>> {
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

    return paths;
}

// Not really a spiral, more like connected circles
fn path_round_spiral(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut x = w as f64 / 2.0;
    let mut y = h as f64 / 2.0;
    let pixelcount = w*h;
    // The max radius has to be
    let max_size = ((w*w + h*h) as f64).sqrt().ceil() as u64;

    let mut r = 1.0;
    let angle_offset = -0.5 * PI;

    // TODO: let angle_offset be set
    // TODO: make elliptic, not just circles
    // TODO: Allow to set center

    let line_path = |r| {
        let mut path = Vec::new();
        let step_amounts = 16 * r as u64;
        let step_size: f64 = 2.0 * PI / step_amounts as f64;
        for step in 0..=step_amounts {
            let angle = angle_offset + step_size * step as f64;
            let xi = x + angle.cos() * r as f64;
            let yi = y + angle.sin() * r as f64;
            if !is_in_bounds(xi as u64,yi as u64,w,h) {continue;}
            path.push(yi as u64 * w + xi as u64);
        }
        path
    };

    // THREADING, WOOO
    let path_iter = (1..max_size/2).into_par_iter().map(line_path);
    paths = vec![path_iter.flatten().collect()];
    return paths;
}


fn path_circles(w: u64, h: u64) -> Vec<Vec<u64>> {
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

    let line_path = |r| {
        let mut path_left = Vec::new();
        let mut path_right = Vec::new();
        let step_amounts = 8 * r as u64;
        let circ_step_size: f64 = PI / step_amounts as f64;
        for step in 0..=step_amounts {
            let angle = angle_offset + circ_step_size * step as f64;
            let xi = x + angle.cos() * r as f64;
            let yi = y + angle.sin() * r as f64;
            path_left.push(yi as u64 * w + xi as u64);
            let angle = angle_offset - circ_step_size * step as f64;
            let xi = x + angle.cos() * r as f64;
            let yi = y + angle.sin() * r as f64;
            path_right.push(yi as u64 * w + xi as u64);
        }
        vec![path_left, path_right]
    };
    // THREADING, WOOO
    let paths = (1..max_size/2).into_par_iter().map(line_path).flatten();

    return paths.collect();
}


/// Creates and returns ranges of mutable Pixels.
/// The picked pixels and their order are determined by the given vector of indices
fn pick_pixels(all_pixels: Vec<&mut Rgb<u8>>, indices: Vec<Vec<u64>>) -> Vec<Vec<&mut Rgb<u8>>> {
    let timestart = Instant::now();
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
    info!("TIME | [Pickin pixels]:\t+ {:?}", timestart.elapsed());

    return paths;
}

fn is_in_bounds(x: u64,y: u64,w: u64,h: u64) -> bool {
    x > 0 && x < w && y > 0 && y < h
}
