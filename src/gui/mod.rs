#![allow(unused)]
use eframe::egui::{
    self, Align, Image, Key, Layout, Modifiers, RichText, TextureFilter, TextureHandle,
    TextureOptions, Ui,
};
use image::RgbImage;
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
    ffi::OsString,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::{AUTHORS, PACKAGE_NAME, VERSION};

mod components;

pub fn start_gui() -> eframe::Result {
    init(None)
}

pub fn start_gui_with_sorter(
    ps: &Pixelsorter,
    img: RgbImage,
    image_path: PathBuf,
) -> eframe::Result {
    init(Some((ps, img, image_path)))
}

fn init(ps: Option<(&Pixelsorter, RgbImage, PathBuf)>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };

    let psgui = if let Some((ps, img, img_path)) = ps {
        PixelsorterGui::from_pixelsorter(ps, img, img_path)
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
    base_img: Option<(RgbImage, PathBuf)>,
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
            base_img: None,
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
    fn from_pixelsorter(ps: &Pixelsorter, img: RgbImage, image_path: PathBuf) -> Self {
        let mut psgui = PixelsorterGui::default();
        psgui.base_img = Some((img, image_path));
        let v = &mut psgui.values;
        v.path = ps.path_creator;
        v.criteria = ps.sorter.criteria;
        v.algorithmn = ps.sorter.algorithm;
        v.reverse = ps.reverse;
        v.selector = ps.selector;
        psgui
    }

    // Reads values and returns a Pixelsorter
    fn to_pixelsorter(&self) -> Pixelsorter {
        let mut ps = Pixelsorter::new();
        ps.path_creator = self.values.path;
        ps.sorter.criteria = self.values.criteria;
        ps.sorter.algorithm = self.values.algorithmn;
        ps.reverse = self.values.reverse;
        ps.selector = self.values.selector;
        ps
    }

    fn sort_img(&self) -> Option<RgbImage> {
        if let Some((img, _)) = &self.base_img {
            let ps = &self.to_pixelsorter();
            let mut img_to_sort = img.clone();
            if (selector_is_threshold(self.values.selector) && self.values.show_mask) {
                ps.mask(&mut img_to_sort);
            } else {
                ps.sort(&mut img_to_sort);
            }
            return Some(img_to_sort);
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
                        self.base_img = Some((img, f));
                        break;
                    }
                    Err(_) => {}
                },
            }
        }
    }

    /// Sorts and saves the image to the current output directory with a given filename
    fn save_file_to_out_dir(&self) -> () {
        if let Some((img, path)) = &self.base_img {
            let mut ps = Pixelsorter::new();
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
        if let Some((_, s)) = &self.base_img {
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
            if i.consume_key(Modifiers::CTRL, Key::S) {
                self.save_file_to_out_dir();
            }
            if i.consume_key(Modifiers::CTRL, Key::O) {
                do_open_file = true;
            }
        });
        if do_open_file {
            self.open_file(ctx);
        }
        // Set default styles
        ctx.style_mut(|style| {
            style.spacing.slider_width = 170.0;
            style.spacing.combo_height = 300.0;
        });

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.horizontal(|ui|{
                ui.label(format!("Time of last sort:\t{:?}", self.time_last_sort));
                ui.label(format!(
                    "({:.3} fps)",
                    (1.0 / self.time_last_sort.as_secs_f32())
                ));
                ui.separator();
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

        let prev_values = self.values.clone();
        egui::SidePanel::left("my-left-pane")
            .resizable(false)
            //.exact_width(380.0)
            .max_width(420.0)
            .show(ctx, |ui| {

                ui.add_space(5.0);

                ui.group(|ui| {
                    ui.set_width(full_width(&ui));
                    if ui.button("Open image...").clicked() {
                        self.open_file(ctx);
                    }

                    if let Some((_, name)) = &self.base_img {
                        ui.label(RichText::new(name.to_string_lossy()));
                    } else {
                        ui.label(RichText::new("No image loaded...").italics());
                    }
                });

                ui.add_space(5.0);

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .max_height(f32::INFINITY)
                        .max_width(f32::INFINITY)
                        .show(ui, |ui| {
                            self.sorting_options_panel(ui, 1);
                        });
                });

                ui.add_space(5.0);

                ui.add_enabled_ui(self.base_img.is_some(), |ui| {
                    // SORT IMAGE button
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

                    ui.add_space(5.0);

                    // SAVING OPTIONS
                    ui.group(|ui| {
                        self.save_options_panel(ui);
                    });

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
