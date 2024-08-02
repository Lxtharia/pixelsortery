#![allow(unused)]
use eframe::egui::{
    self, Align, Image, Layout, RichText, TextureFilter, TextureHandle, TextureOptions, Ui,
};
use image::RgbImage;
use log::{info, warn};
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{FixedSelector, PixelSelectCriteria, RandomSelector, ThresholdSelector},
    span_sorter::{SortingAlgorithm, SortingCriteria},
};
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
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

/// Struct holding all the states of the gui and values of sliders etc.
struct PixelsorterGui {
    /// The image, used to load and be sorted
    img: Option<(RgbImage, PathBuf)>,
    /// The image used by egui to draw every frame
    texture: Option<TextureHandle>,
    /// All the adjustable values for the pixelsorter
    values: PixelsorterValues,
    timestart: Instant,
    time_last_sort: Duration,
    cancel_snd: Sender<()>,
    img_recv: Receiver<RgbImage>,
    threads_started: u32,
}

/// Adjustable components of the pixelsorter
#[derive(Clone, Copy, PartialEq)]
struct PixelsorterValues {
    reverse: bool,
    path: PathCreator,
    selector_type: SelectorType,
    criteria: SortingCriteria,
    algorithmn: SortingAlgorithm,
    /// We can select these with the real structs tbh
    path_diagonally_val: f32,
    selector_random: RandomSelector,
    selector_fixed: FixedSelector,
    selector_thres: ThresholdSelector,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SelectorType {
    Fixed,
    Random,
    Threshold,
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            img: None,
            texture: None,
            values: PixelsorterValues {
                reverse: false,
                path: PathCreator::VerticalLines,
                criteria: SortingCriteria::Brightness,
                selector_type: SelectorType::Threshold,
                algorithmn: SortingAlgorithm::Shellsort,

                path_diagonally_val: 0.0,
                selector_random: RandomSelector { max: 30 },
                selector_fixed: FixedSelector { len: 100 },
                selector_thres: ThresholdSelector {
                    min: 0,
                    max: 360,
                    criteria: PixelSelectCriteria::Hue,
                },
            },
            time_last_sort: Duration::default(),
            cancel_snd: channel().0,
            img_recv: channel().1,
            timestart: Instant::now(),
            threads_started: 0,
        }
    }
}

impl PixelsorterGui {
    fn path_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("path_combo_{}", id))
            .selected_text(format!("{:?}", self.values.path))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::AllHorizontally,
                    "All Horizontally",
                );
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::AllVertically,
                    "All Vertically",
                );
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::HorizontalLines,
                    "Lines, Horizontally",
                );
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::VerticalLines,
                    "Lines, Vertically",
                );
                ui.selectable_value(&mut self.values.path, PathCreator::Circles, "Circles");
                ui.selectable_value(&mut self.values.path, PathCreator::Spiral, "Spiral");
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::SquareSpiral,
                    "Square Spiral",
                );
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::RectSpiral,
                    "Rectangular Spiral",
                );
                ui.selectable_value(
                    &mut self.values.path,
                    PathCreator::Diagonally(self.values.path_diagonally_val),
                    "Diagonally",
                );
            });
        ui.end_row();
        // Path specific tweaks for some pathings
        // Nested Grid for sub-options
        match self.values.path {
            PathCreator::Diagonally(ref mut angle) => {
                ui.label("");
                egui::Grid::new(format!("path_options_grid_{}", id))
                    .num_columns(2)
                    .min_row_height(25.0)
                    .show(ui, |ui| {
                        ui.label("Angle");
                        let slider = egui::Slider::new(angle, 0.0..=360.0)
                            .suffix("°")
                            .clamp_to_range(false)
                            .drag_value_speed(0.2)
                            .max_decimals(1)
                            .smart_aim(false);
                        ui.add(slider);
                        ui.end_row();
                        // Save for when we reselect diagonally
                        self.values.path_diagonally_val = angle.clone();
                    });
                ui.end_row();
            }
            _ => {}
        };
    }

    fn selector_combo_box(&mut self, ui: &mut Ui, id: u64) {
        ui.visuals_mut().weak_text_color();
        egui::ComboBox::from_id_source(format!("selector_combo_{}", id))
            .selected_text(format!("{:?}", self.values.selector_type))
            .show_ui(ui, |ui| {
                vec![
                    SelectorType::Fixed,
                    SelectorType::Random,
                    SelectorType::Threshold,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.values.selector_type, c, format!("{:?}", c));
                });
            });
        ui.end_row();
        // Nested Grid for sub-options
        ui.label("");
        egui::Grid::new(format!("selector_options_grid_{}", id))
            .num_columns(2)
            .min_row_height(25.0)
            .show(ui, |ui| {
                match self.values.selector_type {
                    SelectorType::Fixed => {
                        let len = &mut self.values.selector_fixed.len;
                        ui.label("Length");
                        let slider = egui::Slider::new(len, 0..=2000)
                            .logarithmic(true)
                            .clamp_to_range(false)
                            .drag_value_speed(0.2)
                            .smart_aim(false);
                        ui.add(slider);
                        ui.end_row();
                    }
                    SelectorType::Random => {
                        let max = &mut self.values.selector_random.max;
                        ui.label("Max");
                        let slider = egui::Slider::new(max, 0..=2000)
                            .logarithmic(true)
                            .clamp_to_range(false)
                            .drag_value_speed(0.2)
                            .smart_aim(false)
                            .step_by(1.0);
                        ui.add(slider);
                        ui.end_row();
                    }
                    SelectorType::Threshold => {
                        let min = &mut self.values.selector_thres.min;
                        let max = &mut self.values.selector_thres.max;
                        let criteria = &mut self.values.selector_thres.criteria;

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

                        let (cap, selector_suffix) = if *criteria == PixelSelectCriteria::Hue {
                            (360, "°")
                        } else {
                            (256, "")
                        };

                        ui.label("Min");
                        let min_slider = egui::Slider::new(min, 0..=cap)
                            .trailing_fill(true)
                            .suffix(selector_suffix)
                            .drag_value_speed(0.2)
                            .smart_aim(false);
                        ui.add(min_slider);
                        ui.end_row();

                        ui.label("Max");
                        let max_slider = egui::Slider::new(max, 0..=cap)
                            .trailing_fill(true)
                            .suffix(selector_suffix)
                            .drag_value_speed(0.2)
                            .smart_aim(false);
                        ui.add(max_slider);
                        ui.end_row();

                        // TODO: clamping, depending on which slider is dragged
                    }
                }
            });
        ui.end_row();
    }

    fn criteria_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("criteria_combo_{}", id))
            .selected_text(format!("{:?}", self.values.criteria))
            .show_ui(ui, |ui| {
                vec![
                    SortingCriteria::Brightness,
                    SortingCriteria::Saturation,
                    SortingCriteria::Hue,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.values.criteria, c, format!("{:?}", c));
                });
            });
    }

    fn algorithmn_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_source(format!("algorithm_combo_{}", id))
            .selected_text(format!("{:?}", self.values.algorithmn))
            .show_ui(ui, |ui| {
                vec![
                    SortingAlgorithm::Mapsort,
                    SortingAlgorithm::Glitchsort,
                    SortingAlgorithm::Shellsort,
                    SortingAlgorithm::DebugColor,
                ]
                .into_iter()
                .for_each(|c| {
                    ui.selectable_value(&mut self.values.algorithmn, c, format!("{:?}", c));
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

                ui.checkbox(&mut self.values.reverse, "Reverse?");
                ui.end_row();
            });
    }

    fn sort_img(&mut self) -> () {
        if let Some((img, _)) = self.img.clone() {
            // Cancel ongoing sorts
            self.cancel_snd.send(());
            // Create new cancel channel
            let (canc_snd, canc_rec) = channel();
            self.cancel_snd = canc_snd;
            // Create a channel where we can send the image over
            let (img_snd, img_rec) = channel();
            // Thread
            let mut ps = pixelsortery::Pixelsorter::new(img);
            ps.path_creator = self.values.path;
            ps.sorter.criteria = self.values.criteria;
            ps.sorter.algorithm = self.values.algorithmn;
            ps.reverse = self.values.reverse;
            ps.selector = match self.values.selector_type {
                SelectorType::Fixed => Box::new(self.values.selector_fixed),
                SelectorType::Random => Box::new(self.values.selector_random),
                SelectorType::Threshold => Box::new(self.values.selector_thres),
            };
            self.threads_started += 1;
            thread::spawn(move || {
                ps.sort_cancelable(&canc_rec);
                // Cancle last minute
                if canc_rec.try_recv().is_ok() {
                    return;
                }
                img_snd.send(ps.get_img());
            });
            self.img_recv = img_rec;
            //
        }
    }

    fn set_texture(&mut self, ctx: &egui::Context, img: &RgbImage, name: String) {
        info!("TEX: Received sorted image!");
        let t = Instant::now();
        let rgb_data = img.to_vec();
        let colorimg =
            egui::ColorImage::from_rgb([img.width() as usize, img.height() as usize], &rgb_data);
        let mut options = TextureOptions::default();
        // Make small images stretch without blurring
        options.magnification = TextureFilter::Nearest;
        // Make big images fit without noise
        options.minification = TextureFilter::Linear;
        self.texture = Some(ctx.load_texture(name, colorimg, options));
        info!("TEX: Texture set!! {:?}", t.elapsed());
    }

    /// Tries to show the image if it exists, or not.
    fn show_img(&self, ui: &mut Ui) {
        if let Some(tex) = &self.texture {
            egui::Frame::canvas(ui.style_mut()).show(ui, |ui| {
                ui.add(Image::new((tex.id(), tex.size_vec2())).shrink_to_fit());
            });
        }
    }

    fn open_file(&mut self, ctx: &egui::Context) -> () {
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
                        self.set_texture(ctx, &img, f.to_string_lossy().to_string());
                        self.img = Some((img, f));
                        break;
                    }
                    Err(_) => {}
                },
            }
        }
    }

    /// Sorts and saves the image to a chosen location
    fn save_file(&mut self) -> () {
        if let Some((_, s)) = &self.img {
            let suggested_filename = if let (Some(stem), Some(ext)) = (s.file_stem(), s.extension())
            {
                format!(
                    "{}-sorted.{}",
                    stem.to_string_lossy(),
                    ext.to_string_lossy()
                )
            } else {
                String::from("")
            };
            let file = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .set_file_name(suggested_filename)
                .save_file();
            if let Some(f) = file {
                self.sort_img();
                if let Ok(sorted_img) = self.img_recv.recv() {
                    if let Err(err_msg) = sorted_img.save(&f) {
                        warn!(
                            "Saving image to {} failed: {}",
                            f.to_string_lossy(),
                            err_msg
                        );
                    };
                };
            }
        }
    }
}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let prev_values = self.values.clone();
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
                        self.open_file(ctx);
                    }

                    if let Some((_, name)) = &self.img {
                        ui.label(RichText::new(name.to_string_lossy()));
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
                    // ui.add_space(full_height(ui) - 50.0);

                    ui.separator();

                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        if ui.button(RichText::new("SORT IMAGE").heading()).clicked() {
                            info!("SORTING IMAGE");
                            self.timestart = Instant::now();
                            self.sort_img();
                        }
                    });
                    ui.separator();

                    ui.group(|ui| {
                        ui.set_width(full_width(&ui));
                        // let w = ui.max_rect().max.x - ui.max_rect().min.x;
                        // ui.set_width(w);
                        if ui
                            .add_enabled(self.img.is_some(), egui::Button::new("Save as..."))
                            .clicked()
                        {
                            info!("Saving image...");
                            self.save_file();
                        }
                    });

                    ui.separator();

                    ui.label(format!("Time of last sort:\t{:?}", self.time_last_sort));
                    ui.label(format!("Threads started:\t{:?}", self.threads_started));
                });
            });

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some(tex) = &self.texture {
                    let [w, h] = tex.size();
                    ui.label(format!("{} x {} ({} pixels)", w, h, w * h));
                    ui.separator();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_img(ui);
        });

        // Auto-Sort on changes
        let current_values = self.values;
        if prev_values != current_values {
            println!("Change detected, sorting image...");
            self.timestart = Instant::now();
            self.sort_img();
        }

        ctx.request_repaint();

        // Set received image
        if let Ok(img) = self.img_recv.try_recv() {
            self.time_last_sort = self.timestart.elapsed();
            self.set_texture(ctx, &img, String::from("Image-sorted"));
        }
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
