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
}

impl Default for PixelsorterGui {
    fn default() -> Self {
        Self {
            val: 66,
            s: "lorem ipsum".into(),
        }
    }
}

impl PixelsorterGui {
    fn sorting_options_panel(&mut self, ui: &mut Ui) {
        egui::Grid::new("sorting_option_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
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

                ui.label("Selector");
                if ui.button("Ja/Nein").clicked() {
                    info!("Button has been pressed!");
                }
                ui.end_row();

                ui.label("Sorter");
                ui.text_edit_singleline(&mut self.s);
                ui.end_row();
            });
    }

    fn selector_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.colored_label(Color32::GOLD, "Selector Options");
            ui.add(egui::Slider::new(&mut self.val, 0..=360))
        });
    }
}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::SidePanel::left("my-left-pane")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Left");
                // ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                self.selector_panel(ui);
                ui.separator();
                self.sorting_options_panel(ui);
                ui.separator();
                // });
            });

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
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
