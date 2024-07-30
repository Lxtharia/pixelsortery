#![allow(unused)]
use eframe::egui::{self, Align, Color32, Layout, RichText, Ui};
use log::{debug, info};

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
    val: i64,
    s: String,
    checked: bool,
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            val: 66,
            s: "lorem ipsum".into(),
            checked: false,
        }
    }
}

impl PixelsorterGui {
    fn sorting_options_panel(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.colored_label(Color32::GOLD, "Sorting Options");

            egui::Grid::new("sorting_option_grid")
                .num_columns(2)
                .spacing([30.0, 4.0])
                .min_row_height(25.0)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Path");
                    egui::ComboBox::from_id_source("path_combo")
                        .selected_text("Diagonal")
                        .show_ui(ui, |ui| {
                            ui.selectable_label(false, "Left");
                            ui.selectable_label(true, "Right");
                        });
                    ui.end_row();

                    if self.checked {
                        ui.label("Minimum");
                        ui.add(egui::Slider::new(&mut self.val, 0..=360));
                        ui.end_row();
                        ui.label("Maximum");
                        ui.add(egui::Slider::new(&mut self.val, 0..=360));
                        ui.end_row();
                    }

                    ui.label("Selector");
                    if ui.button("Ja/Nein").clicked() {
                        info!("Button has been pressed!");
                    }
                    ui.end_row();

                    ui.label("Sorter");
                    ui.text_edit_singleline(&mut self.s);
                    ui.end_row();
                    ui.checkbox(&mut self.checked, "Checked?");
                });
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
                    .show(ui, |ui| {
                        // ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        self.sorting_options_panel(ui);
                        self.sorting_options_panel(ui);
                        self.sorting_options_panel(ui);
                        self.sorting_options_panel(ui);
                        // });
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
