#![allow(unused)]
use std::io::Read;

use eframe::egui::{
    self, accesskit::ListStyle, Align, Color32, Image, Layout, Pos2, Rect, RichText, Rounding,
    Style, TextBuffer, TextureFilter, TextureHandle, TextureOptions, Ui,
};
use image::{GenericImageView, RgbImage};
use log::{debug, info};
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{
        FixedSelector, PixelSelectCriteria, PixelSelector, RandomSelector, ThresholdSelector,
    },
    span_sorter::{SortingAlgorithm, SortingCriteria},
};

pub fn start_gui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pixelsortery",
        options,
        Box::new(|cc| {
            // This gives us image support
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<PixelsorterGui>::default())
        }),
    )
}

struct PixelsorterGui {
    /// The image, used to load and be sorted
    img: Option<(RgbImage, String)>,
    /// The image used by egui to draw every frame
    texture: Option<TextureHandle>,
    /// Components of the pixelsorter
    reverse: bool,
    path: PathCreator,
    selector_type: SelectorType,
    selector: Box<dyn PixelSelector>,
    criteria: SortingCriteria,
    algorithmn: SortingAlgorithm,
    /// We can select these with the real structs tbh
    tmp_path_diag_val: f32,
    my_selector_random: RandomSelector,
    my_selector_fixed: FixedSelector,
    my_selector_thres: ThresholdSelector,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectorType {
    Fixed,
    Random,
    Thres,
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            img: None,
            texture: None,
            reverse: false,
            path: PathCreator::VerticalLines,
            criteria: SortingCriteria::Brightness,
            selector_type: SelectorType::Thres,
            selector: Box::new(RandomSelector { max: 30 }),
            algorithmn: SortingAlgorithm::Mapsort,

            tmp_path_diag_val: 0.0,
            my_selector_random: RandomSelector { max: 30 },
            my_selector_fixed: FixedSelector {len: 100},
            my_selector_thres: ThresholdSelector{min: 0, max: 360, criteria: PixelSelectCriteria::Hue},
        }
    }
}

impl PixelsorterGui {
    fn path_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("path_combo_{}", id))
            .selected_text(format!("{:?}", self.path))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.path,
                    PathCreator::AllHorizontally,
                    "All, Horizontally",
                );
                ui.selectable_value(
                    &mut self.path,
                    PathCreator::AllVertically,
                    "All, Vertically",
                );
                ui.selectable_value(
                    &mut self.path,
                    PathCreator::HorizontalLines,
                    "Horizontal Lines",
                );
                ui.selectable_value(&mut self.path, PathCreator::VerticalLines, "Vertical Lines");
                ui.selectable_value(&mut self.path, PathCreator::Circles, "Circles");
                ui.selectable_value(&mut self.path, PathCreator::Spiral, "Spiral");
                ui.selectable_value(&mut self.path, PathCreator::SquareSpiral, "Square Spiral");
                ui.selectable_value(
                    &mut self.path,
                    PathCreator::RectSpiral,
                    "Rectangular Spiral",
                );
                ui.selectable_value(
                    &mut self.path,
                    PathCreator::Diagonally(self.tmp_path_diag_val),
                    "Diagonally",
                );
            });
        ui.end_row();
        // Path specific tweaks for some pathings
        // Nested Grid for sub-options
        match self.path {
            PathCreator::Diagonally(ref mut angle) => {
                ui.label("");
                egui::Grid::new(format!("path_options_grid_{}", id))
                    .num_columns(2)
                    .min_row_height(25.0)
                    .show(ui, |ui| {
                        ui.label("Angle");
                        ui.add(egui::Slider::new(angle, 0.0..=360.0));
                        ui.end_row();
                        // Save for when we reselect diagonally
                        self.tmp_path_diag_val = angle.clone();
                    });
                ui.end_row();
            }
            _ => {}
        };
    }

    fn selector_combo_box(&mut self, ui: &mut Ui, id: u64) {
        ui.visuals_mut().weak_text_color();
        egui::ComboBox::from_id_source(format!("selector_combo_{}", id))
            .selected_text(format!("{:?}", self.selector_type))
            .show_ui(ui, |ui| {
                vec![
                    SelectorType::Fixed,
                    SelectorType::Random,
                    SelectorType::Thres,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.selector_type, c, format!("{:?}", c));
                });
            });
        ui.end_row();
        // Nested Grid for sub-options
        ui.label("");
        egui::Grid::new(format!("selector_options_grid_{}", id))
            .num_columns(2)
            .min_row_height(25.0)
            .show(ui, |ui| {
                match self.selector_type {
                    SelectorType::Fixed => {
                        let len = &mut self.my_selector_fixed.len;
                        ui.label("Length");
                        ui.add(egui::Slider::new(len, 0..=1000));
                        ui.end_row();
                        self.selector = Box::new(FixedSelector { len: *len });
                    }
                    SelectorType::Random => {
                        let max = &mut self.my_selector_random.max;
                        ui.label("Max");
                        ui.add(egui::Slider::new(max, 0..=1000));
                        ui.end_row();
                        self.selector = Box::new(RandomSelector { max: *max });
                    }
                    SelectorType::Thres => {
                        let min = &mut self.my_selector_thres.min;
                        let max = &mut self.my_selector_thres.max;
                        let criteria = &mut self.my_selector_thres.criteria;

                        ui.label("Criteria");
                        egui::ComboBox::from_id_source(format!("selector_criteria_combo_{}", id))
                            .selected_text(format!("{:?}", criteria))
                            .show_ui(ui, |ui| {
                                vec![
                                    PixelSelectCriteria::Hue,
                                    PixelSelectCriteria::Brightness,
                                    PixelSelectCriteria::Saturation,
                                ]
                                .into_iter()
                                .for_each(|c| {
                                    ui.selectable_value(criteria, c, format!("{:?}", c));
                                });
                            });
                        ui.end_row();

                        let cap = if *criteria == PixelSelectCriteria::Hue {
                            360
                        } else {
                            256
                        };

                        ui.label("Lower Bound");
                        ui.add(egui::Slider::new(min, 0..=cap));
                        ui.end_row();

                        ui.label("Upper Bound");
                        ui.add(egui::Slider::new(max, 0..=cap));
                        ui.end_row();

                        // TODO: clamping, depending on which slider is dragged
                        self.selector = Box::new(ThresholdSelector {
                            min: *min,
                            max: *max,
                            criteria: *criteria,
                        });
                    }
                }
            });
        ui.end_row();
    }

    fn criteria_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("criteria_combo_{}", id))
            .selected_text(format!("{:?}", self.criteria))
            .show_ui(ui, |ui| {
                vec![
                    SortingCriteria::Brightness,
                    SortingCriteria::Saturation,
                    SortingCriteria::Hue,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.criteria, c, format!("{:?}", c));
                });
            });
    }

    fn algorithmn_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("algorithm_combo_{}", id))
            .selected_text(format!("{:?}", self.algorithmn))
            .show_ui(ui, |ui| {
                vec![
                    SortingAlgorithm::Mapsort,
                    SortingAlgorithm::Glitchsort,
                    SortingAlgorithm::Shellsort,
                    SortingAlgorithm::DebugColor,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.algorithmn, c, format!("{:?}", c));
                });
            });
    }

    fn sorting_options_panel(&mut self, ui: &mut Ui, id: u64) {
        // ui.vertical_centered(|ui| {
        // ui.colored_label(Color32::GOLD, "Sorting Options");
        // });

        egui::Grid::new(format!("sorting_options_grid_{}", id))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .min_row_height(25.0)
            .striped(true)
            .show(ui, |ui| {
                ui.label("");
                ui.separator();
                ui.end_row();

                // PATH
                ui.label("Path");
                self.path_combo_box(ui, id);

                // SELECTOR
                ui.label("Selector");
                self.selector_combo_box(ui, id);

                // SORTER
                // SORTING CRITERIA
                ui.label("Criteria");
                self.criteria_combo_box(ui, id);
                ui.end_row();
                // SORTING ALGORITHM
                ui.label("Algorithm");
                self.algorithmn_combo_box(ui, id);
                ui.end_row();

                ui.checkbox(&mut self.reverse, "Reverse?");
                ui.end_row();
            });
    }

    fn sort_img(&mut self) -> Option<RgbImage> {
        if let Some((img, _)) = &mut self.img {
            let mut ps = pixelsortery::Pixelsorter::new(img.clone());
            ps.path_creator = self.path;
            ps.sorter.criteria = self.criteria;
            ps.sorter.algorithm = self.algorithmn;
            ps.reverse = self.reverse;
            ps.selector = match self.selector_type {
                SelectorType::Fixed => Box::new(self.my_selector_fixed),
                SelectorType::Random => Box::new(self.my_selector_random),
                SelectorType::Thres => Box::new(self.my_selector_thres),
            };
            ps.sort();
            return Some(ps.get_img());
        }
        return None;
    }

    fn set_texture(&mut self, ctx: &egui::Context, img: &RgbImage, name: String) {
        let rgb_data = img.to_vec();
        let colorimg =
            egui::ColorImage::from_rgb([img.width() as usize, img.height() as usize], &rgb_data);
        let mut options = TextureOptions::default();
        // Make small images stretch without blurring
        options.magnification = TextureFilter::Nearest;
        // Make big images fit without noise
        options.minification = TextureFilter::Linear;
        self.texture = Some(ctx.load_texture("ImageYeahYeah", colorimg, options));
    }

    /// Tries to show the image if it exists, or not.
    fn show_img(&self, ctx: &egui::Context, ui: &mut Ui) {
        if let Some(tex) = &self.texture {
            ui.add(Image::new((tex.id(), tex.size_vec2())).shrink_to_fit());
        }
    }
}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("my-left-pane")
            .resizable(true)
            //.exact_width(380.0)
            .max_width(420.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.separator();
                    ui.heading("Options");
                    ui.separator();
                });

                ui.group(|ui| {
                    ui.set_width(full_width(&ui));
                    if ui.button("Open image...").clicked() {
                        // Opening image until cancled or until valid image loaded
                        loop {
                            let file = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                .pick_file();
                            match file {
                                None => break,
                                Some(f) => match image::open(f.as_path()) {
                                    Ok(i) => {
                                        let img = i.into_rgb8();
                                        self.set_texture(
                                            ctx,
                                            &img,
                                            f.to_string_lossy().to_string(),
                                        );
                                        self.img = Some((img, f.to_string_lossy().to_string()));
                                        break;
                                    }
                                    Err(_) => {}
                                },
                            }
                        }
                    }

                    if let Some((_, name)) = &self.img {
                        ui.label(RichText::new(name).italics());
                    } else {
                        ui.label(RichText::new("No image loaded...").italics());
                    }
                });

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .show(ui, |ui| {
                            self.sorting_options_panel(ui, 1);
                        });
                });

                ui.add_enabled_ui(self.img.is_some(), |ui| {
                    ui.add_space(full_height(ui) - 50.0);

                    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        ui.group(|ui| {
                            // let w = ui.max_rect().max.x - ui.max_rect().min.x;
                            // ui.set_width(w);
                            if ui
                                .add_enabled(self.img.is_some(), egui::Button::new("Save as..."))
                                .clicked()
                            {
                                info!("Saving image...");
                            }
                        });

                        ui.separator();

                        if ui.button(RichText::new("SORT IMAGE").heading()).clicked() {
                            info!("SORTING IMAGE");
                            if let Some(img) = self.sort_img() {
                                self.set_texture(ctx, &img, "Some name".to_string());
                            }
                        }
                    });
                });
            });

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label("Info Panel!");
                ui.separator();
                ui.label(format!("Image Dimensions: {} x {}", 1920, 1080));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello Pixely World!");
            self.show_img(ctx, ui);
        });
    }
}

fn set_full_width(ui: &mut Ui) -> () {
    ui.set_width(ui.max_rect().max.x - ui.max_rect().min.x);
}

fn full_width(ui: &Ui) -> f32 {
    ui.max_rect().max.x - ui.max_rect().min.x
}

fn full_height(ui: &Ui) -> f32 {
    ui.max_rect().max.y - ui.max_rect().min.y
}
