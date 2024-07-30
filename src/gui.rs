#![allow(unused)]
use eframe::egui::{self, Align, Color32, Layout, RichText, Ui};
use log::{debug, info};
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{PixelSelector, RandomSelector},
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
    image_name: Option<String>,
    path: PathCreator,
    tmp_path_diag_val: f32,
    reverse: bool,
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            image_name: None,
            path: PathCreator::HorizontalLines,
            tmp_path_diag_val: 0.0,
            reverse: false,
        }
    }
}

impl PixelsorterGui {
    fn sorting_options_panel(&mut self, ui: &mut Ui, id: u64) {
        // ui.vertical_centered(|ui| {
        // ui.colored_label(Color32::GOLD, "Sorting Options");
        // });

        egui::Grid::new(format!("sorting_options_grid_{}", id))
            .num_columns(2)
            .spacing([30.0, 4.0])
            .min_row_height(25.0)
            .striped(true)
            .show(ui, |ui| {
                ui.label("");
                ui.separator();
                ui.end_row();
                // PATH
                ui.label("Path");
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
                        ui.selectable_value(
                            &mut self.path,
                            PathCreator::VerticalLines,
                            "Vertical Lines",
                        );
                        ui.selectable_value(&mut self.path, PathCreator::Circles, "Circles");
                        ui.selectable_value(&mut self.path, PathCreator::Spiral, "Spiral");
                        ui.selectable_value(
                            &mut self.path,
                            PathCreator::SquareSpiral,
                            "Square Spiral",
                        );
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
                match self.path {
                    PathCreator::Diagonally(ref mut angle) => {
                        ui.label("Angle");
                        ui.add(egui::Slider::new(angle, 0.0..=360.0));
                        ui.end_row();
                        angle.clamp(0.0, 360.0);
                        // Save for when we reselect diagonally
                        self.tmp_path_diag_val = angle.clone();
                    }
                    _ => {}
                };

                // SORTER
                // SORTING CRITERIA
                ui.label("Criteria");
                ui.end_row();
                // SORTING ALGORITHM
                ui.label("Algorithm");
                ui.end_row();

                ui.checkbox(&mut self.reverse, "Reverse?");
                ui.end_row();
            });
    }
}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("my-left-pane")
            .resizable(false)
            .exact_width(380.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Options");
                    ui.separator();
                });

                egui::ScrollArea::vertical()
                    .max_height(f32::INFINITY)
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        self.sorting_options_panel(ui, 1);
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
        });
    }
}
