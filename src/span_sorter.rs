use image::Rgb;

use crate::color_helpers;
mod glitchsort;
mod mapsort;
mod random_color;
mod shellsort;

#[derive(Debug)]
pub struct SpanSorter {
    pub criteria: SortingCriteria,
    pub algorithm: SortingAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingCriteria {
    Hue,
    Brightness,
    Saturation,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingAlgorithm {
    Mapsort,
    Shellsort,
    Glitchsort,
    DebugColor,
}

impl SpanSorter {
    // Create new SpanSorter with sorting criteria and algorithm
    pub fn new(criteria: SortingCriteria) -> SpanSorter {
        SpanSorter {
            criteria,
            algorithm: SortingAlgorithm::Mapsort,
        }
    }
    pub fn info_string(&self) -> String {
        format!("Sorting pixel intervals by [{:?}] using [{:?}]", self.criteria, self.algorithm)
    }

    // Set criteria of SpanSorter
    pub fn set_criteria(&mut self, criteria: SortingCriteria) {
        self.criteria = criteria;
    }

    /// Returns the function that extracts the value we sort by.
    ///
    /// For example when sorting by Hue, this returns the function that calculates the hue of a pixel.
    pub fn get_value_function(criteria: SortingCriteria) -> for<'a> fn(&'a Rgb<u8>) -> u16 {
        match criteria {
            SortingCriteria::Brightness => color_helpers::get_brightness,
            SortingCriteria::Saturation => color_helpers::get_saturation,
            SortingCriteria::Hue | _ => color_helpers::get_hue,
        }
    }

    // Choose fitting algorithm for criteria
    // Idk why. This is dumb.
    pub fn determine_algorithm(&mut self) {
        self.algorithm = match self.criteria {
            SortingCriteria::Hue => SortingAlgorithm::Mapsort,
            SortingCriteria::Brightness => SortingAlgorithm::Mapsort,
            SortingCriteria::Saturation => SortingAlgorithm::Mapsort,
        };
    }

    /// Sort a slice of pixels using set criteria and algorithm
    pub fn sort(&self, pixels: &mut [&mut Rgb<u8>]) {
        // Select function per algorithm
        let sorting_function = match self.algorithm {
            SortingAlgorithm::DebugColor => random_color::set_random_color,
            SortingAlgorithm::Mapsort => mapsort::mapsort_mut,
            SortingAlgorithm::Shellsort => shellsort::shellsort_mut,
            SortingAlgorithm::Glitchsort => glitchsort::glitchsort_mut,
        };
        // Use a special, flawed brightness function for glitchsorting
        let criteria_function = match (self.algorithm, self.criteria) {
            (SortingAlgorithm::Glitchsort, SortingCriteria::Brightness) => color_helpers::get_brightness_flawed,
            _ => SpanSorter::get_value_function(self.criteria),
        };
        match self.algorithm {
            // Apply debug color even on every span
            SortingAlgorithm::DebugColor => sorting_function(pixels, criteria_function),
            // Skip sorting a span if it contains less than 2 pixels
            _ => if pixels.len() < 2 {return;},
        }
        // call sorting function
        sorting_function(pixels, criteria_function);
    }
}
