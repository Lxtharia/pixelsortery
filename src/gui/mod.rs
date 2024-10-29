#![allow(unused)]
use eframe::egui::{
    self, Align, Button, Color32, Image, Key, Layout, Modifiers, RichText, ScrollArea,
    TextureFilter, TextureHandle, TextureOptions, Ui, Vec2,
};
use egui::{scroll_area::ScrollBarVisibility, style::ScrollStyle};
use egui_flex::{item, Flex, FlexAlign, FlexAlignContent, FlexJustify};
use image::RgbImage;
use inflections::case::to_title_case;
use layers::LayeredSorter;
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
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{AUTHORS, PACKAGE_NAME, VERSION};
mod layers;

/// How long the "Saved" Label should be visible before it vanishes
const SAVED_LABEL_VANISH_TIMEOUT: f32 = 2.0;
const INITIAL_WINDOW_SIZE: Vec2 = egui::vec2(1000.0, 700.0);

mod components;

pub fn init(ps: Option<&Pixelsorter>, img: Option<(RgbImage, PathBuf)>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(INITIAL_WINDOW_SIZE),
        ..Default::default()
    };

    let mut psgui = PixelsorterGui::default();
    if let Some(ps) = ps {
        psgui = psgui.with_values(ps);
    }
    if let Some((img, img_path)) = img {
        psgui = psgui.with_image(img, img_path);
    }

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
    /// The path of the loaded image
    path: Option<PathBuf>,
    /// The Values and sorted Images, layered
    layered_sorter: Option<LayeredSorter>,
    /// All the adjustable values for the pixelsorter
    values: PixelsorterValues,
    show_mask: bool,
    /// The current image from the selected layer
    img: Option<RgbImage>,
    /// The image used by egui to draw every frame
    texture: Option<TextureHandle>,
    output_directory: Option<PathBuf>,
    save_into_parent_dir: bool,
    time_last_sort: Duration,
    auto_sort: bool,
    do_sort: bool,
    saving_success_timeout: Option<Instant>,
    change_layer: SwitchLayerMessage,
    show_base_image: bool,
}

/// Adjustable components of the pixelsorter, remembers values like diagonal angle
#[derive(Clone, Copy, PartialEq)]
struct PixelsorterValues {
    reverse: bool,
    path: PathCreator,
    selector: PixelSelector,
    criteria: SortingCriteria,
    algorithm: SortingAlgorithm,
    // Values that may not be set right now, but the values should be remembered
    /// We can select these with the real structs tbh
    path_diagonally_val: f32,
    selector_random: PixelSelector,
    selector_fixed: PixelSelector,
    selector_thres: PixelSelector,
}

#[derive(PartialEq)]
enum SwitchLayerMessage {
    None,
    BaseImage,
    Layer(usize),
    DeleteLayer(usize),
}

impl PixelsorterValues {
    // Reads values and returns a Pixelsorter
    fn to_pixelsorter(&self) -> Pixelsorter {
        let mut ps = Pixelsorter::new();
        ps.path_creator = self.path;
        ps.selector = self.selector;
        ps.sorter.criteria = self.criteria;
        ps.sorter.algorithm = self.algorithm;
        ps.reverse = self.reverse;
        ps
    }

    fn read_from_pixelsorter(&mut self, ps: &Pixelsorter) {
        self.path = ps.path_creator;
        self.selector = ps.selector;
        self.criteria = ps.sorter.criteria;
        self.algorithm = ps.sorter.algorithm;
        self.reverse = ps.reverse;
        // Set the saved value, just in case
        if let PathCreator::Diagonally(a) = self.path {
            self.path_diagonally_val = a;
        }
        match self.selector {
            Fixed { len } => self.selector_fixed = Fixed { len },
            Random { max } => self.selector_random = Random { max },
            Threshold { min, max, criteria } => {
                self.selector_thres = Threshold { min, max, criteria }
            }
            Full => warn!("The gui doesn't support the Full-Selector. Just because."),
        }
    }
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            path: None,
            layered_sorter: None,
            img: None,
            texture: None,
            show_mask: false,
            values: PixelsorterValues {
                reverse: false,
                path: PathCreator::VerticalLines,
                criteria: SortingCriteria::Brightness,
                selector: PixelSelector::Threshold {
                    min: 0,
                    max: 360,
                    criteria: PixelSelectCriteria::Hue,
                },
                algorithm: SortingAlgorithm::Shellsort,

                path_diagonally_val: 45.0,
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
            saving_success_timeout: None,
            change_layer: SwitchLayerMessage::None,
            do_sort: true,
            show_base_image: false,
        }
    }
}

impl PixelsorterGui {
    // Given a pixelsorter, return a PixelsorterGui with the values set
    fn with_values(mut self, ps: &Pixelsorter) -> Self {
        self.values.path = ps.path_creator;
        self.values.criteria = ps.sorter.criteria;
        self.values.algorithm = ps.sorter.algorithm;
        self.values.reverse = ps.reverse;
        self.values.selector = ps.selector;
        self
    }
    fn with_image(mut self, img: RgbImage, image_path: PathBuf) -> Self {
        self.img = Some(img);
        self.path = Some(image_path);
        self
    }

    /// Calls sort_current_layer, sets the image and texture
    fn sort_img(&mut self, ctx: &egui::Context, force: bool) {
        if let Some(ls) = &mut self.layered_sorter {
            // values.selector is not up-to-date with the current layer as it seems
            if selector_is_threshold(ls.get_current_layer().get_sorting_values().selector)
                && self.show_mask
            {
                // We can unwrap here, because the mask() function only fails if we don't have a threshold selector
                self.img = Some(ls.get_mask_for_current_layer().unwrap().clone());
            } else {
                let timestart = Instant::now();
                let did_sort = if force {
                    ls.sort_current_layer();
                    true
                } else {
                    ls.sort_current_layer_cached()
                };
                // Set the time only when it actually sorted something (because it might decide that it doesnt need to sort)
                if did_sort {
                    self.time_last_sort = timestart.elapsed();
                }
                self.img = Some(ls.get_current_layer().get_img().clone());
            }
        }
        // Display sorted image
        self.update_texture(ctx);
    }

    /// Update texture
    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.img {
            let rgb_data = img.to_vec();
            let colorimg = egui::ColorImage::from_rgb(
                [img.width() as usize, img.height() as usize],
                &rgb_data,
            );
            let mut options = TextureOptions::default();
            // Make small images stretch without blurring
            options.magnification = TextureFilter::Nearest;
            // Make big images fit without noise
            options.minification = TextureFilter::Linear;
            self.texture = Some(ctx.load_texture("Some texture name", colorimg, options));
        }
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
                        if let Some(ls) = &mut self.layered_sorter {
                            ls.set_base_img(img.clone());
                        }
                        // I want to make layered_sorter mandatory and remove the possibility of it being None
                        self.img = Some(img);
                        self.update_texture(ctx);
                        self.path = Some(f);
                        break;
                    }
                    Err(_) => {}
                },
            }
        }
    }

    /// Sorts and saves the image to the current output directory with a given filename
    fn save_file_to_out_dir(&mut self) -> () {
        if let Some(path) = &self.path {
            let mut ps = Pixelsorter::new();
            ps.path_creator = self.values.path;
            ps.sorter.criteria = self.values.criteria;
            ps.sorter.algorithm = self.values.algorithm;
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
                if outpath.is_dir() {
                    outpath.push(filename);
                } else {
                    outpath.set_file_name(filename);
                }
                if let Some(sorted) = &self.img {
                    if let Ok(p) = save_image(sorted, &outpath) {
                        info!("Saved file to '{}' ...", p.to_string_lossy());
                        self.saving_success_timeout = Some(Instant::now());
                    }
                }
            } else {
                warn!("No output directory set");
            }
        }
    }

    /// Sorts and saves the image to a chosen location
    fn save_file_as(&mut self) -> () {
        if let Some(p) = &self.path {
            let suggested_filename = if let (Some(stem), Some(ext)) = (p.file_stem(), p.extension())
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
                if let Some(sorted) = &self.img {
                    if let Err(err_msg) = sorted.save(&f) {
                        warn!(
                            "Saving image to {} failed: {}",
                            f.to_string_lossy(),
                            err_msg
                        );
                    } else {
                        info!("Saved file to '{}' ...", f.to_string_lossy());
                        self.saving_success_timeout = Some(Instant::now());
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
            if i.consume_key(Modifiers::CTRL, Key::S) {
                self.save_file_to_out_dir();
            }
            if i.consume_key(Modifiers::CTRL, Key::O) {
                do_open_file = true;
            }
            if i.consume_key(Modifiers::NONE, Key::M) {
                self.show_mask = !self.show_mask;
                self.do_sort = true;
            }
            if i.consume_key(Modifiers::NONE, Key::Questionmark) {
                self.layered_sorter.as_ref().unwrap().print_state();
            }
        });
        // Set default styles
        ctx.style_mut(|style| {
            style.spacing.slider_width = 170.0;
            style.spacing.combo_height = 300.0;
        });

        // Open file on startup
        if do_open_file {
            self.open_file(ctx);
        }
        if let Some(ls) = &self.layered_sorter {
            // Load current values
            self.values = ls.get_current_layer().get_sorting_values().clone();
        } else {
            // Create a layering thingy if we don't have one yet
            if let Some(img) = &self.img {
                self.layered_sorter = Some(LayeredSorter::new(img.clone(), self.values));
            }
        }

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Time of last sort:\t{:?}", self.time_last_sort));
                ui.label(format!(
                    "({:.3} fps)",
                    (1.0 / self.time_last_sort.as_secs_f32())
                ));
                ui.separator();
                if let Some(inst) = self.saving_success_timeout {
                    if inst.elapsed() > Duration::from_secs_f32(SAVED_LABEL_VANISH_TIMEOUT) {
                        self.saving_success_timeout = None;
                    } else {
                        ui.label(RichText::new("Saved!").color(Color32::DARK_GREEN));
                        ui.separator();
                    }
                }
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
        });

        let prev_values = (self.values.clone(), self.show_mask);
        egui::SidePanel::left("my-left-pane")
            .resizable(false)
            //.exact_width(380.0)
            .max_width(380.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.group(|ui| {
                            ui.set_width(full_width(&ui));
                            if ui.button("Open image...").clicked() {
                                self.open_file(ctx);
                            }

                            if let Some(p) = &self.path {
                                ui.label(RichText::new(p.to_string_lossy()));
                            } else {
                                ui.label(RichText::new("No image loaded...").italics());
                            }
                        });

                        ui.add_space(5.0);

                        // SLIDERS AND BUTTONS AND STUFF
                        ui.group(|ui| {
                            self.sorting_options_panel(ui, 1);
                        });

                        ui.add_space(5.0);

                        ui.add_enabled_ui(self.img.is_some(), |ui| {
                            // SORT IMAGE button
                            ui.columns(3, |columns| {
                                let ui = &mut columns[0];
                                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                                    ui.checkbox(&mut self.auto_sort, "Auto-sort");
                                });
                                let ui = &mut columns[1];
                                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                                    if ui.button(RichText::new("SORT IMAGE").heading()).clicked() {
                                        self.do_sort = true;
                                    }
                                });
                                let ui = &mut columns[2];
                                ui.add_enabled_ui(
                                    selector_is_threshold(self.values.selector),
                                    |ui| {
                                        ui.checkbox(&mut self.show_mask, "Show mask");
                                    },
                                );
                            });
                        });

                        ui.add_space(5.0);

                        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                            ui.add_space(5.0);
                            // SAVING OPTIONS
                            ui.add_enabled_ui(self.img.is_some(), |ui| {
                                ui.group(|ui| {
                                    self.save_options_panel(ui);
                                });
                            });
                            ui.add_space(5.0);
                            ui.group(|ui| {
                                // LAYERING
                                ui.style_mut().spacing.scroll.floating = false;
                                ScrollArea::vertical()
                                    .id_salt("LayerScrollArea")
                                    .show(ui, |ui| {
                                        Flex::vertical()
                                            .align_content(FlexAlignContent::Stretch)
                                            .wrap(false)
                                            .justify(FlexJustify::End)
                                            .show(ui, |flex| {
                                                flex.grow();
                                                //for i in 0..8 {
                                                //    let b = Button::new(format!("LMAO: {}", i));
                                                //    flex.add(item().basis(30.0), b);
                                                //}
                                                self.layering_panel(flex);
                                            });
                                    });
                            });
                        });
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_img(ui);
        });
        if let Some(ls) = &mut self.layered_sorter {
            // info!("Setting values for current: {}", self.values.to_pixelsorter().to_compact_string());
            // Write any changes back to the layered sorter
            ls.update_current(self.values.clone());
        }

        // Auto-Sort current image on changes or if image needs sorting
        let values_changed = self.values != prev_values.0 || self.show_mask != prev_values.1;
        if (self.do_sort || (self.auto_sort && values_changed)) {
            self.do_sort = false;
            self.sort_img(&ctx, true);
        }

        //
        if let Some(ls) = &mut self.layered_sorter {
            // We are switching layers!!
            // Sort after switching
            if let SwitchLayerMessage::Layer(i) = self.change_layer {
                self.change_layer = SwitchLayerMessage::None;
                ls.select_layer(i);
                self.show_base_image = false;
                self.sort_img(ctx, false);
                ctx.request_repaint();
            } else if SwitchLayerMessage::BaseImage == self.change_layer {
                self.change_layer = SwitchLayerMessage::None;
                self.show_base_image = true;
                self.img = Some(ls.get_base_img().clone());
                self.update_texture(ctx);
            } else if let SwitchLayerMessage::DeleteLayer(i) = self.change_layer {
                self.change_layer = SwitchLayerMessage::None;
                // We are deleting the current selected layer
                if i == ls.get_current_index() && !self.show_base_image {
                    ls.remove_layer(i);
                    // Trigger a select on the (now) current layer (remove_layer may have reselected one)
                    self.change_layer = SwitchLayerMessage::Layer(ls.get_current_index());
                } else {
                    // We are deleting another layer than the current one
                    // Or we are looking at the base image
                    ls.remove_layer(i);
                }
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

/// Saves image to a path. If the file already exists, increment the name.
/// Returns the path the image finally gets saved to
/// Returns an Error if the path is a directory
pub fn save_image(img: &RgbImage, path: &PathBuf) -> Result<PathBuf, String> {
    if path.is_dir() {
        return Err(String::from("Destination is a directory"));
    }
    let org_stem = path.file_stem().unwrap_or_default().to_os_string();
    let org_ext = path.extension().unwrap_or_default().to_os_string();
    let mut new_path = path.clone();

    let mut counter = 1;
    while let Ok(true) = new_path.try_exists() {
        counter += 1;
        let mut new_name = OsString::new();
        new_name.push(&org_stem);
        new_name.push(format!("-{}.", counter));
        new_name.push(&org_ext);
        new_path.set_file_name(new_name);
    }
    if let Err(e) = img.save(&new_path) {
        warn!(
            "Saving image to '{}' failed: {}",
            new_path.to_string_lossy(),
            e
        );
        return Err(e.to_string());
    }
    Ok(new_path)
}
