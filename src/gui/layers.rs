use image::RgbImage;
use pixelsortery::Pixelsorter;

use super::PixelsorterValues;

pub(crate) struct LayeredSorter {
    base_img: RgbImage,
    layers: Vec<SortingLayer>,
    current_layer: usize,
}

pub(crate) struct SortingLayer {
    sorting_values: PixelsorterValues,
    sorted_img: RgbImage,
    needs_sorting: bool,
}

impl LayeredSorter {
    pub(crate) fn new(img: RgbImage, ps: PixelsorterValues) -> Self {
        let mut ls = LayeredSorter {
            base_img: img,
            layers: Vec::new(),
            current_layer: 0,
        };
        ls.add_layer(ps);
        ls
    }

    pub(crate) fn get_layers(&self) -> &Vec<SortingLayer> {
        &self.layers
    }
    pub(crate) fn get_layer<T: Into<usize>>(&self, ind: T) -> Option<&SortingLayer> {
        self.layers.get(ind.into())
    }
    pub(crate) fn get_current_layer(&self) -> &SortingLayer {
        self.layers.get(self.current_layer).unwrap()
    }
    pub(crate) fn get_current_index(&self) -> usize {
        self.current_layer
    }
    pub(crate) fn get_base_img(&self) -> &RgbImage {
        &self.base_img
    }
    pub(crate) fn set_base_img(&mut self, img: RgbImage) {
        self.base_img = img;
        self.invalidate_layers_above(0usize);
    }

    pub(crate) fn print_state(&self) {
        println!("Printing Layers");
        for (i, l) in self.layers.iter().enumerate() {
            println!(
                "[{}] {} {} {}",
                i,
                if l.needs_sorting { "*" } else { " " },
                l.sorting_values.to_pixelsorter().to_compact_string(),
                if i == self.current_layer { "<<<<" } else { "" },
            );
        }
    }

    /// Adds a layer, but don't sort it
    pub(crate) fn add_layer(&mut self, ps: PixelsorterValues) {
        let prev_img = if let Some(last) = self.get_layers().last() {
            last.get_img()
        } else {
            &self.base_img
        };
        // Creates a new layer at the end (which will need sorting)
        let layer = SortingLayer::new(ps, prev_img.clone());
        self.layers.push(layer);
    }

    /// Set values. Will determien if sort is needed
    pub(crate) fn update_current(&mut self, values: PixelsorterValues) {
        &mut self
            .layers
            .get_mut(self.current_layer)
            .unwrap()
            .set_sorting_values(values);
    }

    /// Removes the layer and selects the one below, or the one above if unavailable
    pub(crate) fn remove_layer<T: Into<usize>>(&mut self, ind: T) -> bool {
        let ind = ind.into();
        // Don't allow removing if we only have one layer left
        if self.layers.len() <= 1 || ind >= self.layers.len() {
            return false;
        }

        // If the last layer was selected, adjust the selected index
        if self.current_layer == self.layers.len() - 1 || ind < self.current_layer {
            self.current_layer -= 1;
        }

        self.invalidate_layers_above(ind);
        self.layers.remove(ind);
        true
    }

    pub(crate) fn select_layer<T: Into<usize>>(&mut self, ind: T) -> Option<&SortingLayer> {
        let ind = ind.into();
        if ind < self.layers.len() {
            self.current_layer = ind;
            Some(&self.layers.get(ind).unwrap())
        } else {
            None
        }
    }

    /// Marks all layers above index (exclusive) that they need sorting
    pub(crate) fn invalidate_layers_above<T: Into<usize>>(&mut self, ind: T) {
        let ind = ind.into();
        for layer in self.layers.iter_mut().skip(ind + 1).rev() {
            layer.needs_sorting = true;
        }
    }

    /// Sorts the layer and all layers below if nessesary. Also marks all layers above as "needs-sorting"
    fn sort_layer<T: Into<usize>>(&mut self, ind: T) -> bool {
        let ind = ind.into();
        if ind >= self.layers.len() {
            return false;
        }

        let mut prev_img = &self.base_img;
        // As long as the layers are up to date, we don't need to sort
        let mut needs_sorting = false;
        for (i, layer) in self.layers.iter_mut().take(ind + 1).enumerate() {
            // Once we needed to sort a layer, we need to sort all of them
            needs_sorting = needs_sorting || layer.needs_sorting;
            // Sort the layer at [index] in any case
            if i == ind || needs_sorting {
                layer.sort(&prev_img);
            }
            prev_img = &layer.sorted_img;
        }
        self.invalidate_layers_above(ind);
        true
    }

    /// Sort the current layer, regardless if it's already sorted or not
    pub(crate) fn sort_current_layer(&mut self) {
        self.sort_layer(self.current_layer);
    }

    /// Sorts the current layer if it needs sorting
    pub(crate) fn sort_current_layer_cached(&mut self) -> bool {
        if self.get_current_layer().needs_sorting {
            self.sort_current_layer();
            true
        } else {
            false
        }
    }

    /// Sorts all layers below if needed but will not sort the current one, but instead only show the mask
    pub(crate) fn get_mask_for_current_layer(&mut self) -> Option<RgbImage> {
        let prev_index = self.current_layer - 1;
        self.sort_layer(prev_index);
        let prev_img = if let Some(layer) = self.get_layer(prev_index) {
            layer.get_img()
        } else {
            &self.base_img
        };
        self.get_current_layer().get_mask(prev_img)
    }
}

impl SortingLayer {
    /// Creates a new layer with an unsorted image (the one below it in the best case) and mark it as needs_sorting
    pub(crate) fn new(ps: PixelsorterValues, img: RgbImage) -> Self {
        SortingLayer {
            sorting_values: ps,
            sorted_img: img,
            needs_sorting: true,
        }
    }

    pub(crate) fn get_sorting_values(&self) -> &PixelsorterValues {
        &self.sorting_values
    }
    pub(crate) fn set_sorting_values(&mut self, ps: PixelsorterValues) {
        // Only set as needs_sorting when the new values are actually different?
        if self.sorting_values != ps {
            self.needs_sorting = true;
        }
        self.sorting_values = ps;
    }

    pub(crate) fn get_img(&self) -> &RgbImage {
        &self.sorted_img
    }

    fn sort(&mut self, img: &RgbImage) {
        let mut sorted_img = img.clone();
        self.sorting_values.to_pixelsorter().sort(&mut sorted_img);
        self.sorted_img = sorted_img;
        self.needs_sorting = false;
    }

    fn get_mask(&self, img: &RgbImage) -> Option<RgbImage> {
        let mut masked_img = img.clone();
        if self.sorting_values.to_pixelsorter().mask(&mut masked_img) {
            return Some(masked_img);
        }
        None
    }
}
