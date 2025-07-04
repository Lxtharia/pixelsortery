use egui::TextBuffer;
use image::{Rgb, RgbImage};
use log::{error, info, warn};
use rayon::prelude::*;
use std::{f64::consts::PI, fmt::Display, time::Instant};

mod gilbert;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCreator {
    AllHorizontally,
    AllVertically,
    HorizontalLines,
    VerticalLines,
    Rays,
    Circles,
    Spiral,
    SquareSpiral,
    RectSpiral,
    Diagonally(f32),
    Hilbert,
}

impl std::fmt::Display for PathCreator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PathCreator::AllHorizontally => "All Horizontally".into(),
                PathCreator::AllVertically => "All Vertically".into(),
                PathCreator::HorizontalLines => "Left/Right".into(),
                PathCreator::VerticalLines => "Up/Down".into(),
                PathCreator::Rays => "Rays".into(),
                PathCreator::Circles => "Circles".into(),
                PathCreator::Spiral => "Spiral".into(),
                PathCreator::SquareSpiral => "Square Spiral".into(),
                PathCreator::RectSpiral => "Rectangular Spiral".into(),
                PathCreator::Diagonally(a) => format!("Diagonally ({}°)", a),
                PathCreator::Hilbert => "Hilbert Curve".into(),
            }
        )
    }
}

impl PathCreator {
    pub fn info_string(self) -> String {
        format!("Direction/Order: [{:?}]", self)
    }
    pub fn create_paths(self, all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64, reverse: bool) -> Vec<Vec<&mut Rgb<u8>>> {

        let mut total_timestart = Instant::now();
        let mut timestart = Instant::now();

        // Actual path algorithms
        // Ideas/missing:
        // In waves
        // In star shape
        let mut all_paths_indices = match self {
            PathCreator::AllHorizontally => path_all_horizontally(w, h),
            PathCreator::AllVertically => path_all_vertically(w, h),
            PathCreator::HorizontalLines => path_horizontal_lines(w, h),
            PathCreator::VerticalLines => path_vertical_lines(w, h),
            PathCreator::Rays => path_rays(w, h),
            PathCreator::SquareSpiral => path_rect_spiral(w, h, true),
            PathCreator::RectSpiral => path_rect_spiral(w, h, false),
            PathCreator::Diagonally(angle) => path_diagonal_lines(w, h, angle),
            PathCreator::Circles => path_circles(w, h),
            PathCreator::Spiral => path_round_spiral(w, h),
            PathCreator::Hilbert => gilbert::path_hilbert(w, h),
        };
        let timeend_pathing = timestart.elapsed();
        timestart = Instant::now();

        // Reverse spans if nessesary
        if reverse {
            all_paths_indices.iter_mut().for_each(|p| {
                p.reverse();
            });
        }
        let timeend_reversing = timestart.elapsed();
        timestart = Instant::now();

        // Turn indexed paths into arrays of pixels
        let pixels = pick_pixels(all_pixels, all_paths_indices);
        let timeend_picking = timestart.elapsed();

        info!("TIME | [Index Pathing]:  \t+ {:?}", timeend_pathing);
        info!("TIME | [Reversing paths]:\t+ {:?}", timeend_reversing);
        info!("TIME | [Pickin pixels]:  \t+ {:?}", timeend_picking);
        info!(
            "TIME | [Creating Paths]: \t= {:?}",
            total_timestart.elapsed()
        );

        return pixels;
    }
}

/// Creates and returns ranges of mutable Pixels.
/// The picked pixels and their order are determined by the given vector of indices
fn pick_pixels(all_pixels: Vec<&mut Rgb<u8>>, indices: Vec<Vec<u64>>) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
    let mut all_pixels: Vec<Option<&mut Rgb<u8>>> =
        all_pixels.into_iter().map(|p| Some(p)).collect();

    for mut li in indices {
        li.dedup();
        let mut path = Vec::new();
        for i in li {
            // Check if the index is valid
            if all_pixels.get(i as usize).is_some() {
                all_pixels.push(None);
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

fn is_in_bounds(x: u64, y: u64, w: u64, h: u64) -> bool {
    x > 0 && x < w && y > 0 && y < h
}

fn path_all_horizontally(w: u64, h: u64) -> Vec<Vec<u64>> {
    vec![(0..w * h).collect()]
}

fn path_all_vertically(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut path = Vec::new();

    for x in 0..w {
        for y in 0..h {
            let i = y * w + x;
            path.push(i);
        }
    }

    return vec![path];
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
        paths.push((0..h).into_iter().map(|y| y * w + x).collect());
    }

    return paths;
}

fn path_diagonal_lines(w: u64, h: u64, angle: f32) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    // Explanation:
    // We need to iterate differently, depending on the angle
    //
    // x_line_path (a=160°)  x_line_path: (a=110°)          y_line_path: (a=110°)
    //
    //   0  1  2              0 1 2 3 4 5 6 7               0 1 2 3 4 5 6 7
    // 0 X        <Good     0 X                           0 X X
    // 1 X                  1     X                       1     X X
    // 2    X        Bad>   2         X        Correct>   2         X X
    // 3    X               3             X               3             X X
    let angle = angle % 360.0;
    let do_reverse = (angle > 45.0 && angle < 225.0) || (angle < -135.0 && angle > -315.0);
    let angle = angle % 180.0;
    let x_tan_val = angle.to_radians().tan();
    let xoverhead = (x_tan_val * h as f32).round() as i64;
    let xrange = if x_tan_val < 0.0 {
        xoverhead..w as i64
    } else {
        0..w as i64 + xoverhead
    };

    let x_line_path = |xs| {
        let mut path = Vec::new();
        for y in 0..h {
            let x = xs - (y as f32 * x_tan_val).round() as i64;
            // Prevent "overflowing" the index and selecting indices on the next line
            if x >= w as i64 || x < 0 {
                continue;
            }
            let i = y * w + x as u64;
            path.push(i);
        }
        path
    };

    let y_tan_val = (90.0 - angle).to_radians().tan();
    let yoverhead = (y_tan_val * w as f32).round() as i64;
    let yrange = if y_tan_val < 0.0 {
        yoverhead..h as i64
    } else {
        0..h as i64 + yoverhead
    };

    let y_line_path = |ys| {
        let mut path = Vec::new();
        for x in 0..w {
            let y = ys - (x as f32 * y_tan_val).round() as i64;
            // Prevent "overflowing" the index and selecting indices on the next line
            if y >= h as i64 || y < 0 {
                continue;
            }
            let i = y as u64 * w + x;
            path.push(i);
        }
        path
    };

    // Choosing the correct function
    let mut paths: Vec<Vec<u64>> = match (angle.abs() > 45.0 && angle.abs() < 135.0) {
        // THREADPOOLING WOOO
        true => yrange.into_iter().map(y_line_path).collect(),
        false => xrange.into_iter().map(x_line_path).collect(),
    };
    if (do_reverse) {
        for p in &mut paths {
            p.reverse()
        }
    }

    return paths;
}

// Helper function to return a range (including the end element), but in a binary-search-like order
fn spread_range(start: u64, end: u64) -> Vec<u64> {
    if start == end { return vec![end]; }
    if start.abs_diff(end) <= 1 { return vec![start, end]; }
    let mid = (end + start) / 2;
    let mut v = vec![mid];
    let left = spread_range(start, mid-1);
    let right = spread_range(mid+1, end);
    //println!("{{{}}} {:?} {} {:?} {{{}}}", start, left, mid, right, end);
    v.extend_from_slice(&left);
    v.extend_from_slice(&right);
    return v;
}


fn path_rays(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut cx:f32 = (w as f32) / 2.0;
    let mut cy:f32 = (h as f32) / 2.0;
    let mut dirs: Vec<(f32, f32)> = Vec::new();;
    let mut tips: Vec<(f32, f32)>;
    let  mut y = 0; let mut x = 0;
    // Uses the spread range to generate "full" rays first to prevent all rays from being uncontinuous because of overlap
    tips =      spread_range(0, w-1).into_iter().map(|x| { (x as f32, 0.0) }).collect();
    tips.extend(spread_range(0, w-1).into_iter().map(|x| { (x as f32, h as f32) }));
    tips.extend(spread_range(0, h-1).into_iter().map(|y| { (0.0, y as f32) }));
    tips.extend(spread_range(0, h-1).into_iter().map(|y| { (w as f32, y as f32) }));
    dirs = tips.into_iter().map(|(x,y)| {
        let dx = x - cx;
        let dy = y - cy;
        let m = 2.0 * dx.abs().max(dy.abs());
        (dx/m, dy/m)
    }).collect();
    let ray = |(dx, dy): (f32, f32)| {
        let mut x = cx;
        let mut y = cy;
        let mut path = Vec::new();
        loop {
            x += dx;
            y += dy;
            let i: u64 = y.round() as u64 * w + x.round() as u64;
            path.push(i);
            if x < 0.0 || x.ceil() > w as f32  || y < 0.0 || y.ceil() > h as f32 { break; }
        }
        path
    };

    return dirs.into_par_iter().map(ray).collect();
}

fn path_rect_spiral(w: u64, h: u64, square: bool) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut x = w / 2;
    let mut y = h / 2;
    let pixelcount = w * h;
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
        if w > h {
            reach_x = std::cmp::max(1, w - h);
        } else {
            reach_y = std::cmp::max(1, h - w);
        }
        x -= reach_x / 2;
        y -= reach_y / 2;
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
    let pixelcount = w * h;
    // The max radius has to be
    let max_size = ((w * w + h * h) as f64).sqrt().ceil() as u64;

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
            if !is_in_bounds(xi as u64, yi as u64, w, h) {
                continue;
            }
            path.push(yi as u64 * w + xi as u64);
        }
        path
    };

    // THREADING, WOOO
    let path_iter = (1..max_size / 2).into_par_iter().map(line_path);
    paths = vec![path_iter.flatten().collect()];
    return paths;
}

fn path_circles(w: u64, h: u64) -> Vec<Vec<u64>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();
    let mut x = w as f64 / 2.0;
    let mut y = h as f64 / 2.0;
    let pixelcount = w * h;
    // The max radius has to be
    let max_size = ((w * w + h * h) as f64).sqrt().ceil() as u64;

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
    paths.par_extend((1..max_size / 2).into_par_iter().map(line_path).flatten());
    return paths;
}
