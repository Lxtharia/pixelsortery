use image::Rgb;
mod random_color;
mod mapsort;
mod glitchsort;

#[derive(Debug)]
pub struct SpanSorter {
    pub criteria: SortingCriteria,
    pub algorithm: SortingAlgorithm,
    // inverse: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingCriteria {
    Hue,
    Brightness,
    Saturation,
    Debug,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingAlgorithm {
    Mapsort,
    Glitchsort,
    DebugColor,
}

impl SpanSorter {
    // Create new SpanSorter with sorting criteria and algorithm
    pub fn new(criteria: SortingCriteria) -> SpanSorter {
        SpanSorter{criteria, algorithm: SortingAlgorithm::Mapsort}
    }
    // Set criteria of SpanSorter
    pub fn set_criteria(&mut self, criteria: SortingCriteria){
        self.criteria = criteria;
    }

    // Choose fitting algorithm for criteria
    // Idk why. This is dumb.
    pub fn determine_algorithm(&mut self){
        self.algorithm = match self.criteria {
            SortingCriteria::Debug      => SortingAlgorithm::DebugColor,
            SortingCriteria::Hue        => SortingAlgorithm::Mapsort,
            SortingCriteria::Brightness => SortingAlgorithm::Mapsort,
            SortingCriteria::Saturation => SortingAlgorithm::Mapsort,
            _ => SortingAlgorithm::Mapsort,
        };
    }

    // Sort slice of pixels using set criteria and algorithm
    pub fn sort(&self, pixels: &mut [&mut Rgb<u8>]) {
        // Select function per algorithm
        let sorting_function = match self.algorithm {
            SortingAlgorithm::DebugColor => random_color::set_random_color,
            SortingAlgorithm::Mapsort => mapsort::mapsort_mut,
            SortingAlgorithm::Glitchsort => glitchsort::glitchsort_mut,
            _ => random_color::set_random_color,
        };
        // call sorting function
        sorting_function(pixels, &self.criteria);
    }
}

