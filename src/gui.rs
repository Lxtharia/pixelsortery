#![allow(unused)]
use eframe::{
    egui::{
        self, style::HandleShape, Align, Image, Key, Layout, Modifiers, RichText, TextureFilter,
        TextureHandle, TextureOptions, Ui,
    },
    epaint::Hsva,
};
use image::{Pixel, RgbImage};
use inflections::case::to_title_case;
use log::{info, warn};
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{
        PixelSelectCriteria,
        PixelSelector::{self, *},
    },
    span_sorter::{SortingAlgorithm, SortingCriteria},
    Pixelsorter,
};
use std::{
    borrow::Cow,
    ffi::OsString,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::{AUTHORS, PACKAGE_NAME, VERSION};

pub fn start_gui() -> eframe::Result {
    init(None)
}

pub fn start_gui_with_sorter(ps: &Pixelsorter, image_path: PathBuf) -> eframe::Result {
    init(Some((ps, image_path)))
}

fn init(ps: Option<(&Pixelsorter, PathBuf)>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    let psgui = if let Some((ps, img_path)) = ps {
        PixelsorterGui::from_pixelsorter(ps, img_path)
    } else {
        PixelsorterGui::default()
    };

    eframe::run_native(
        "Pixelsortery",
        options,
        Box::new(|cc| {
            // This gives us image support
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(psgui))
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
    output_directory: Option<PathBuf>,
    save_into_parent_dir: bool,
    time_last_sort: Duration,
    auto_sort: bool,
}

/// Adjustable components of the pixelsorter
#[derive(Clone, Copy, PartialEq)]
struct PixelsorterValues {
    show_mask: bool,
    reverse: bool,
    path: PathCreator,
    selector: PixelSelector,
    criteria: SortingCriteria,
    algorithmn: SortingAlgorithm,
    /// We can select these with the real structs tbh
    path_diagonally_val: f32,
    selector_random: PixelSelector,
    selector_fixed: PixelSelector,
    selector_thres: PixelSelector,
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            img: None,
            texture: None,
            values: PixelsorterValues {
                show_mask: false,
                reverse: false,
                path: PathCreator::VerticalLines,
                criteria: SortingCriteria::Brightness,
                selector: PixelSelector::Threshold {
                    min: 0,
                    max: 360,
                    criteria: PixelSelectCriteria::Hue,
                },
                algorithmn: SortingAlgorithm::Shellsort,

                path_diagonally_val: 0.0,
                selector_random: PixelSelector::Random { max: 30 },
                selector_fixed: PixelSelector::Fixed { len: 100 },
                selector_thres: PixelSelector::Threshold {
                    min: 0,
                    max: 360,
                    criteria: PixelSelectCriteria::Brightness,
                },
            },
            time_last_sort: Duration::default(),
            auto_sort: true,
            output_directory: None,
            save_into_parent_dir: false,
        }
    }
}

impl PixelsorterGui {
    // Given a pixelsorter, return a PixelsorterGui with the values set
    fn from_pixelsorter(ps: &Pixelsorter, image_path: PathBuf) -> Self {
        let mut psgui = PixelsorterGui::default();
        psgui.img = Some((ps.get_img(), image_path));
        let v = &mut psgui.values;
        v.path = ps.path_creator;
        v.criteria = ps.sorter.criteria;
        v.algorithmn = ps.sorter.algorithm;
        v.reverse = ps.reverse;
        v.selector = ps.selector;
        psgui
    }

    // Reads values and returns a Pixelsorter
    fn to_pixelsorter(&self) -> Option<Pixelsorter> {
        if let Some((img, _)) = &self.img {
            let mut ps = pixelsortery::Pixelsorter::new(img.clone());
            ps.path_creator = self.values.path;
            ps.sorter.criteria = self.values.criteria;
            ps.sorter.algorithm = self.values.algorithmn;
            ps.reverse = self.values.reverse;
            ps.selector = self.values.selector;
            Some(ps)
        } else {
            None
        }
    }

    fn path_combo_box(&mut self, ui: &mut Ui, id: u64) {
        let path_name_mappings = [
            (PathCreator::AllHorizontally, "All Horizontally"),
            (PathCreator::AllVertically, "All Vertically"),
            (PathCreator::HorizontalLines, "Left/Right"),
            (PathCreator::VerticalLines, "Up/Down"),
            (PathCreator::Circles, "Circles"),
            (PathCreator::Spiral, "Spiral"),
            (PathCreator::SquareSpiral, "Square Spiral"),
            (PathCreator::RectSpiral, "Rectangular Spiral"),
            (
                PathCreator::Diagonally(self.values.path_diagonally_val),
                &format!("Diagonally ({}°)", self.values.path_diagonally_val),
            ),
            (PathCreator::Hilbert, "Hilbert Curve"),
        ];
        // The text that's shown in the combobox, debug as default
        let path_debug_name = &format!("{:?}", &self.values.path);
        let selected_text = match path_name_mappings
            .into_iter()
            .find(|x| x.0 == self.values.path)
        {
            Some((_, t)) => t,
            None => path_debug_name,
        };

        ui.horizontal(|ui| {
            // Build ComboBox from entries in the path_name_mappings
            egui::ComboBox::from_id_source(format!("path_combo_{}", id))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (v, t) in path_name_mappings {
                        ui.selectable_value(&mut self.values.path, v, t);
                    }
                });

            // Reverse checkbox
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.checkbox(&mut self.values.reverse, "Reverse?");
            });
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
        ui.horizontal(|ui| {
            for (s, n) in vec![
                (self.values.selector_fixed, "Fixed"),
                (self.values.selector_random, "Random"),
                (self.values.selector_thres, "Threshold"),
            ] {
                ui.selectable_value(&mut self.values.selector, s, n);
            }
        });
        ui.end_row();

        // Nested Grid for sub-options
        ui.label("");
        egui::Grid::new(format!("selector_options_grid_{}", id))
            .num_columns(2)
            .min_row_height(25.0)
            .show(ui, |ui| {
                match &mut self.values.selector {
                    PixelSelector::Fixed { len } => {
                        ui.label("Length");
                        let slider = egui::Slider::new(len, 0..=2000)
                            .logarithmic(true)
                            .clamp_to_range(false)
                            .drag_value_speed(0.2)
                            .smart_aim(false);
                        ui.add(slider);
                        ui.end_row();
                        // Save selector state
                        self.values.selector_fixed = self.values.selector;
                    }
                    PixelSelector::Random { max } => {
                        ui.label("Max");
                        let slider = egui::Slider::new(max, 0..=2000)
                            .logarithmic(true)
                            .clamp_to_range(false)
                            .drag_value_speed(0.2)
                            .smart_aim(false)
                            .step_by(1.0);
                        ui.add(slider);
                        ui.end_row();
                        // Save selector state
                        self.values.selector_random = self.values.selector;
                    }
                    PixelSelector::Threshold { min, max, criteria } => {
                        ui.label("Criteria");
                        ui.horizontal(|ui| {
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

                        // Get slider colors and image
                        // HSVA::new(hue, saturation, brightness, alpha)
                        let (mincol, maxcol, criteria_image) = match criteria {
                            PixelSelectCriteria::Hue => (
                                Hsva::new(*min as f32 / 360.0, 1.0, 1.0, 1.0).into(),
                                Hsva::new(*max as f32 / 360.0, 1.0, 1.0, 1.0).into(),
                                Image::new(egui::include_image!("../assets/hue-bar.png")),
                            ),
                            PixelSelectCriteria::Brightness => (
                                Hsva::new(1.0, 0.0, *min as f32 / 256.0, 1.0).into(),
                                Hsva::new(1.0, 0.0, *max as f32 / 256.0, 1.0).into(),
                                Image::new(egui::include_image!("../assets/brightness-bar.png")),
                            ),
                            PixelSelectCriteria::Saturation => (
                                Hsva::new(1.0, *min as f32 / 256.0, 1.0, 1.0).into(),
                                Hsva::new(1.0, *max as f32 / 256.0, 1.0, 1.0).into(),
                                Image::new(egui::include_image!("../assets/saturation-bar.png")),
                            ),
                        };

                        ui.label("Min");
                        ui.scope(|ui| {
                            ui.style_mut().visuals.selection.bg_fill = mincol;
                            let min_slider = egui::Slider::new(min, 0..=cap)
                                .handle_shape(HandleShape::Rect { aspect_ratio: 0.5 })
                                .trailing_fill(true)
                                .suffix(selector_suffix)
                                .drag_value_speed(0.2)
                                .smart_aim(false);
                            if ui.add(min_slider).dragged() {
                                *max = (*max).clamp(*min, u64::MAX);
                            };
                        });
                        ui.end_row();

                        ui.label("");
                        ui.add(
                            criteria_image
                                .maintain_aspect_ratio(false)
                                .fit_to_exact_size([ui.style().spacing.slider_width, 15.0].into()),
                        );
                        ui.end_row();

                        ui.label("Max");
                        ui.scope(|ui| {
                            ui.style_mut().visuals.selection.bg_fill = maxcol;
                            let max_slider = egui::Slider::new(max, 0..=cap)
                                .handle_shape(HandleShape::Rect { aspect_ratio: 0.5 })
                                .trailing_fill(true)
                                .suffix(selector_suffix)
                                .drag_value_speed(0.2)
                                .smart_aim(false);

                            if ui.add(max_slider).dragged() {
                                *min = (*min).clamp(u64::MIN, *max);
                            };
                        });
                        ui.end_row();

                        // Save selector state
                        self.values.selector_thres = self.values.selector;
                    }
                    // We don't expose the Full Selector to the gui, so I don't wanna support it
                    PixelSelector::Full => self.values.selector = PixelsorterGui::default().values.selector,
                }
            });
        ui.end_row();
    }

    fn criteria_combo_box(&mut self, ui: &mut Ui, id: u64) {
        ui.horizontal(|ui| {
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
                .for_each(|a| {
                    ui.selectable_value(&mut self.values.algorithmn, a, format!("{:?}", a));
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

                ui.label("");
                ui.add_enabled_ui(selector_is_threshold(self.values.selector), |ui| {
                    ui.checkbox(&mut self.values.show_mask, "Show mask");
                });
                ui.end_row();
            });
    }

    fn sort_img(&self) -> Option<RgbImage> {
        if let Some(ps) = &mut self.to_pixelsorter() {
            if (selector_is_threshold(self.values.selector) && self.values.show_mask) {
                ps.mask();
            } else {
                ps.sort();
            }
            return Some(ps.get_img());
        }
        return None;
    }

    /// loads the image as a texture into context
    fn set_texture(&mut self, ctx: &egui::Context, img: &RgbImage, name: String) {
        let rgb_data = img.to_vec();
        let colorimg =
            egui::ColorImage::from_rgb([img.width() as usize, img.height() as usize], &rgb_data);
        let mut options = TextureOptions::default();
        // Make small images stretch without blurring
        options.magnification = TextureFilter::Nearest;
        // Make big images fit without noise
        options.minification = TextureFilter::Linear;
        self.texture = Some(ctx.load_texture(name, colorimg, options));
    }

    /// Tries to show the image if it exists
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

    /// Sorts and saves the image to the current output directory with a given filename
    fn save_file_to_out_dir(&self) -> () {
        if let Some((img, path)) = &self.img {
            let mut ps = pixelsortery::Pixelsorter::new(img.clone());
            ps.path_creator = self.values.path;
            ps.sorter.criteria = self.values.criteria;
            ps.sorter.algorithm = self.values.algorithmn;
            ps.reverse = self.values.reverse;
            ps.selector = self.values.selector;

            let (basename, ext) = (
                path.file_stem()
                    .map(|s| s.to_os_string())
                    .unwrap_or(OsString::from("img")),
                path.extension()
                    .map(|s| s.to_os_string())
                    .unwrap_or(OsString::from("png")),
            );
            let filename = format!(
                "{}_sorted_[{}].{}",
                basename.to_string_lossy(),
                ps.to_compact_string(),
                ext.to_string_lossy()
            );

            let outpath = if self.save_into_parent_dir {
                Some(path.clone())
            } else {
                self.output_directory.clone()
            };

            if let Some(mut outpath) = outpath {
                outpath.set_file_name(filename);
                if let Some(sorted) = self.sort_img() {
                    if let Err(err_msg) = sorted.save(&outpath) {
                        warn!(
                            "Saving image to {} failed: {}",
                            outpath.to_string_lossy(),
                            err_msg
                        );
                    } else {
                        info!("Saving file to '{}' ...", outpath.to_string_lossy());
                    };
                }
            } else {
                warn!("No output directory set");
            }
        }
    }

    /// Sorts and saves the image to a chosen location
    fn save_file_as(&self) -> () {
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
                if let Some(sorted) = self.sort_img() {
                    if let Err(err_msg) = sorted.save(&f) {
                        warn!(
                            "Saving image to {} failed: {}",
                            f.to_string_lossy(),
                            err_msg
                        );
                    };
                }
            }
        }
    }
}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut do_open_file = false;
        // Shortcuts
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::S) {
                self.save_file_as();
            }
            if i.consume_key(Modifiers::CTRL, Key::O) {
                do_open_file = true;
            }
        });
        if do_open_file {
            self.open_file(ctx);
        }
        ctx.style_mut(|style| {
            style.spacing.slider_width = 170.0;
        });

        let prev_values = self.values.clone();
        egui::SidePanel::left("my-left-pane")
            .resizable(false)
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

                    ui.columns(3, |columns| {
                        let ui = &mut columns[1];
                        ui.with_layout(Layout::top_down(Align::Center), |ui| {
                            if ui.button(RichText::new("SORT IMAGE").heading()).clicked() {
                                info!("SORTING IMAGE");
                                let timestart = Instant::now();
                                if let Some(img) = self.sort_img() {
                                    self.time_last_sort = timestart.elapsed();
                                    self.set_texture(ctx, &img, "Some name".to_string());
                                }
                            }
                        });
                        let ui = &mut columns[2];
                        ui.checkbox(&mut self.auto_sort, "Auto-sort");
                    });
                    ui.separator();

                    // SAVING OPTIONS
                    ui.add_enabled_ui(self.img.is_some(), |ui| {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.set_width(full_width(&ui));
                                // let w = ui.max_rect().max.x - ui.max_rect().min.x;
                                // ui.set_width(w);
                                if ui.button("Save as...").clicked() {
                                    info!("Saving image...");
                                    self.save_file_as();
                                }

                                // Enable Save button, if a dir is set, or if we are saving into the parent directory
                                ui.add_enabled_ui(
                                    self.output_directory.is_some() || self.save_into_parent_dir,
                                    |ui| {
                                        if ui.button("Save").clicked() {
                                            info!("Saving image...");
                                            self.save_file_to_out_dir();
                                        }
                                    },
                                );
                                if ui.button("Choose destination...").clicked() {
                                    // TODO filepick
                                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                        self.output_directory = Some(dir);
                                        self.save_into_parent_dir = false;
                                    } else {
                                    }
                                }
                                ui.checkbox(&mut self.save_into_parent_dir, "Same directory");
                            });
                            ui.horizontal(|ui| {
                                ui.group(|ui| {
                                    ui.label("Saving into: ");
                                    let text = if self.save_into_parent_dir {
                                        let mut parent_dir = self.img.as_ref().unwrap().1.clone();
                                        parent_dir.pop();
                                        RichText::new(parent_dir.to_string_lossy()).monospace()
                                    } else if let Some(output_dir) = &self.output_directory {
                                        RichText::new(output_dir.to_string_lossy()).monospace()
                                    } else {
                                        RichText::new("No output directory set").italics()
                                    };
                                    ui.label(text);
                                });
                            });
                        });
                    });

                    ui.separator();

                    ui.label(format!("Time of last sort:\t{:?}", self.time_last_sort));
                    ui.label(format!(
                        "Frames per second:\t{:.3} fps",
                        (1.0 / self.time_last_sort.as_secs_f32())
                    ));
                });
            });

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Max), |ui| {
                ui.label(format!(
                    "{} v{} by {}",
                    to_title_case(PACKAGE_NAME),
                    VERSION,
                    AUTHORS
                ));
                ui.separator();
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
        if self.auto_sort && prev_values != current_values {
            let timestart = Instant::now();
            info!("[Change detected, sorting image...]");
            if let Some(img) = self.sort_img() {
                self.time_last_sort = timestart.elapsed();
                self.set_texture(ctx, &img, String::from("Image-sorted"));
            }
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
fn selector_is_threshold(sel: PixelSelector) -> bool {
    matches!(
        sel,
        Threshold {
            min: _,
            max: _,
            criteria: _
        }
    )
}
