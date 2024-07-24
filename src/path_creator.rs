use image::{Rgb, RgbImage};

#[derive(Debug, Clone, Copy)]
pub enum PathCreator {
    All,
    Horizontal,
    Vertical,
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
            PathCreator::All => path_all,
            PathCreator::Horizontal => path_horizontal,
            PathCreator::Vertical => path_vertical,
        };
        if reverse {
            all_pixels.reverse();
        }
        pathing_function(all_pixels, w, h)
    }
}

fn path_all(all_pixels: Vec<&mut Rgb<u8>>, _: u64, _: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    vec![all_pixels]
}

fn path_horizontal(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
    let mut paths: Vec<Vec<u64>> = Vec::new();

    for y in 0..h {
        paths.push((y*w..y*w+w).collect());
    }

    return pick_pixels(all_pixels, paths);
}

fn path_vertical(all_pixels: Vec<&mut Rgb<u8>>, w: u64, h: u64) -> Vec<Vec<&mut Rgb<u8>>> {
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
            if all_pixels.get(i as usize).is_some() {
                path.push(all_pixels.swap_remove(i as usize).unwrap());
            }
        }
        paths.push(path);
    }

    return paths;
}
