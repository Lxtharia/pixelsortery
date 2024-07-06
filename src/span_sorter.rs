use image::Rgb;
mod random_color;
mod mapsort;

#[derive(Debug)]
pub struct SpanSorter {
    pub method: SortingMethod,
    pub algorithm: SortingAlgorithm,
    // inverse: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingMethod {
    Hue,
    Brightness,
    Saturation,
    Debug,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingAlgorithm {
    Mapsort,
    DebugColor,
}

impl SpanSorter {
    // Create new SpanSorter with sorting criteria and algorithm
    pub fn new() -> SpanSorter {
        SpanSorter{method: SortingMethod::Hue, algorithm: SortingAlgorithm::Mapsort}
    }
    // Set method of SpanSorter
    pub fn set_method(&mut self, method: SortingMethod){
        self.method = method;
    }

    // Choose fitting algorithm for criteria
    // Idk why. This is dumb.
    pub fn determine_algorithm(&mut self){
        self.algorithm = match self.method {
            SortingMethod::Debug      => SortingAlgorithm::DebugColor,
            SortingMethod::Hue        => SortingAlgorithm::Mapsort,
            SortingMethod::Brightness => SortingAlgorithm::Mapsort,
            SortingMethod::Saturation => SortingAlgorithm::Mapsort,
            _ => SortingAlgorithm::Mapsort,
        };
    }

    // Sort slice of pixels using set criteria and algorithm
    pub fn sort(&self, pixels: &mut [&mut Rgb<u8>]) {
        // Select function per algorithm
        let sorting_function = match self.algorithm {
            SortingAlgorithm::DebugColor => random_color::set_random_color,
            SortingAlgorithm::Mapsort => mapsort::mut_map_sort,
            _ => random_color::set_random_color,
        };
        // call sorting function
        sorting_function(pixels, &self.method);
    }
}

