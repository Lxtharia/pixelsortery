use image::RgbImage;
use pixelsortery::Pixelsorter;

struct LayeredSorter {
    base_img: RgbImage,
    layers: Vec<SortingLayer>,
    current_layer: usize,
}

struct SortingLayer {
    sorter: Pixelsorter,
    sorted_img: RgbImage,
    needs_sorting: bool,
}

impl LayeredSorter {
    fn new(img: RgbImage, ps: Pixelsorter) -> Self {
        let mut ls = LayeredSorter {
            base_img: img,
            layers: Vec::new(),
            current_layer: 0,
        };
        ls.add_layer(ps);
        ls
    }

    fn get_layers(&self) -> &Vec<SortingLayer> {
        &self.layers
    }
    fn get_layer(&self, ind: u32) -> Option<&SortingLayer> {
        self.layers.get(ind as usize)
    }
    fn get_selected_layer(&self) -> Option<&SortingLayer> {
        self.layers.get(self.current_layer)
    }

    fn print_state(&self) {
        println!("Printing Layers");
        for (i, l) in self.layers.iter().enumerate() {
            println!("[{}] {} {}", i, l.needs_sorting, l.sorter.to_compact_string());
        }
    }

    /// Adds a layer, but don't sort it
    fn add_layer(&mut self, ps: Pixelsorter) {
        let layer = SortingLayer::new(ps);
        // layer.sort(&self.base_img);
        self.layers.push(layer);
    }

    fn remove_layer(&mut self, ind: u32) -> bool {
        let ind = ind as usize;
        if ind < self.layers.len() {
            self.invalidate_layers_above(ind);
            self.layers.remove(ind);
            true
        } else {
            false
        }
    }

    fn select_layer(&mut self, ind: u32) -> Option<&SortingLayer> {
        let ind = ind as usize;
        if ind < self.layers.len() {
            self.current_layer = ind;
            Some(&self.layers.get(ind).unwrap())
        } else {
            None
        }
    }

    /// Marks all layers above index (exclusive) that they need sorting
    fn invalidate_layers_above<T: Into<usize>>(&mut self, ind: T) {
        let ind = ind.into();
        for layer in self.layers.iter_mut().skip(ind+1).rev() {
            layer.needs_sorting = true;
        }
    }

    fn sort_current_layer(&mut self) {
        self.sort_layer(self.current_layer);
    }

    /// Sorts the layer and all layers below if nessesary. Also marks all layers above as "needs-sorting"
    fn sort_layer<T: Into<usize>>(&mut self, ind: T) -> bool {
        let ind = ind.into();
        if ind >= self.layers.len() {
            return false
        }

        let mut prev_img = &self.base_img;
        // As long as the layers are up to date, we don't need to sort
        let mut up_do_date = true;
        for (i, layer) in self.layers.iter_mut().take(ind+1).enumerate() {
            // Once we needed to sort a layer, we need to sort all of them
            up_do_date = up_do_date || layer.needs_sorting;
            // Sort the layer at [index] in any case
            if i == ind || ! up_do_date {
                layer.sort(&prev_img);
            }
            prev_img = &layer.sorted_img;
        }
        self.invalidate_layers_above(ind);
        true
    }
}

impl SortingLayer {
    // I don't like this tbh
    fn new(ps: Pixelsorter) -> Self {
        SortingLayer {
            sorter: ps,
            sorted_img: RgbImage::new(0, 0),
            needs_sorting: true,
        }
    }

    pub fn get_img(&self) -> &RgbImage {
        &self.sorted_img
    }

    fn sort(&mut self, img: &RgbImage) {
        let mut sorted_img = img.clone();
        self.sorter.sort(&mut sorted_img);
        self.sorted_img = sorted_img;
        self.needs_sorting = false;
    }
}

#[test]
fn allofem() {
    let mut ps = Pixelsorter::new();
    ps.sorter.algorithm = pixelsortery::span_sorter::SortingAlgorithm::Glitchsort;
    ps.path_creator = pixelsortery::path_creator::PathCreator::HorizontalLines;

    let mut ps2 = Pixelsorter::new();
    ps2.selector = pixelsortery::pixel_selector::PixelSelector::Fixed { len: 200 };

    let mut ps3 = Pixelsorter::new();
    ps3.path_creator = pixelsortery::path_creator::PathCreator::Hilbert;

    let bytes = include_bytes!("../assets/test.png");
    let img = image::load_from_memory(bytes).unwrap().into_rgb8();
    let mut ls = LayeredSorter::new(img, ps3);

    ls.add_layer(ps2.clone());
    ls.add_layer(ps);
    
    ls.print_state();

    ls.remove_layer(1);
    ls.add_layer(ps2);

    ls.print_state();

    ls.select_layer(0);
    ls.select_layer(2);

    ls.print_state();

}
