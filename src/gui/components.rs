use eframe::{
    egui::{
        self, style::HandleShape, vec2, Align, Button, Image, RichText, ScrollArea,
        SelectableLabel, Ui,
    },
    epaint::Hsva,
};
use egui::{Align2, Frame, Label, SliderClamping, Stroke, TextBuffer};
use egui_flex::FlexInstance;
use log::info;
use pixelsortery::{
    path_creator::PathCreator,
    pixel_selector::{PixelSelectCriteria, PixelSelector},
    span_sorter::{SortingAlgorithm, SortingCriteria},
};

use super::*;

// A style i want to apply to multiple labels
fn important_text(s: &str) -> RichText {
    //RichText::new(s).underline()
    RichText::new(s).size(14.0)
}

impl PixelsorterGui {
    pub(super) fn path_combo_box(&mut self, ui: &mut Ui, id: u64) {
        let available_paths = vec![
            PathCreator::AllHorizontally,
            PathCreator::AllVertically,
            PathCreator::HorizontalLines,
            PathCreator::VerticalLines,
            PathCreator::Rays,
            PathCreator::Circles,
            PathCreator::Spiral,
            PathCreator::SquareSpiral,
            PathCreator::RectSpiral,
            PathCreator::Diagonally(self.values.path_diagonally_val),
            PathCreator::Hilbert,
        ];
        let selected_text = self.values.path.to_string();

        ui.horizontal(|ui| {
            // Build ComboBox from entries in the path_name_mappings
            egui::ComboBox::from_id_salt(format!("path_combo_{}", id))
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for p in available_paths {
                        ui.selectable_value(&mut self.values.path, p, p.to_string());
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
                        ui.label(important_text("Angle"));
                        let slider = egui::Slider::new(angle, 0.0..=360.0)
                            .suffix("°")
                            .clamping(SliderClamping::Edits)
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

    pub(super) fn selector_combo_box(&mut self, ui: &mut Ui, id: u64) {
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
                        ui.label(important_text("Length"));
                        let slider = egui::Slider::new(len, 0..=2000)
                            .logarithmic(true)
                            .clamping(SliderClamping::Never)
                            .drag_value_speed(0.2)
                            .smart_aim(false);
                        ui.add(slider);
                        ui.end_row();
                        // Save selector state
                        self.values.selector_fixed = self.values.selector;
                    }
                    PixelSelector::Random { max } => {
                        ui.label(important_text("Max"));
                        let slider = egui::Slider::new(max, 0..=2000)
                            .logarithmic(true)
                            .clamping(SliderClamping::Never)
                            .drag_value_speed(0.2)
                            .smart_aim(false)
                            .step_by(1.0);
                        ui.add(slider);
                        ui.end_row();
                        // Save selector state
                        self.values.selector_random = self.values.selector;
                    }
                    PixelSelector::Threshold { min, max, criteria } => {
                        ui.label(important_text("Criteria"));
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
                                Image::new(egui::include_image!("../../assets/hue-bar.png")),
                            ),
                            PixelSelectCriteria::Brightness => (
                                Hsva::new(1.0, 0.0, *min as f32 / 256.0, 1.0).into(),
                                Hsva::new(1.0, 0.0, *max as f32 / 256.0, 1.0).into(),
                                Image::new(egui::include_image!("../../assets/brightness-bar.png")),
                            ),
                            PixelSelectCriteria::Saturation => (
                                Hsva::new(1.0, *min as f32 / 256.0, 1.0, 1.0).into(),
                                Hsva::new(1.0, *max as f32 / 256.0, 1.0, 1.0).into(),
                                Image::new(egui::include_image!("../../assets/saturation-bar.png")),
                            ),
                        };

                        ui.label(important_text("Min"));
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

                        ui.label(important_text("Max"));
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
                    PixelSelector::Full => {
                        self.values.selector = PixelsorterGui::default().values.selector
                    }
                }
            });
        ui.end_row();
    }

    pub(super) fn criteria_combo_box(&mut self, ui: &mut Ui, id: u64) {
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

    pub(super) fn algorithmn_combo_box(&mut self, ui: &mut Ui, id: u64) {
        egui::ComboBox::from_id_salt(format!("algorithm_combo_{}", id))
            .selected_text(format!("{:?}", self.values.algorithm))
            .show_ui(ui, |ui| {
                vec![
                    SortingAlgorithm::Mapsort,
                    SortingAlgorithm::Shellsort,
                    SortingAlgorithm::Glitchsort,
                    SortingAlgorithm::DebugColor,
                ]
                .into_iter()
                .for_each(|a| {
                    ui.selectable_value(&mut self.values.algorithm, a, format!("{:?}", a));
                });
            });
    }

    pub(super) fn sorting_options_panel(&mut self, ui: &mut Ui, id: u64) {
        // ui.vertical_centered(|ui| {
        // ui.colored_label(Color32::GOLD, "Sorting Options");
        // });

        egui::Grid::new(format!("sorting_options_grid_{}", id))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .min_row_height(25.0)
            .striped(true)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Options").size(16.0));
                });
                ui.separator();
                ui.end_row();

                // PATH
                ui.label(important_text("Path"));
                self.path_combo_box(ui, id);

                // SELECTOR
                ui.label(important_text("Selector"));
                self.selector_combo_box(ui, id);

                // SORTER
                // SORTING ALGORITHM
                ui.label(important_text("Algorithm"));
                self.algorithmn_combo_box(ui, id);
                ui.end_row();
                // SORTING CRITERIA
                ui.label(important_text("Criteria"));
                ui.horizontal(|ui| {
                    ui.label("by");
                    self.criteria_combo_box(ui, id);
                });
                ui.end_row();
            });
    }

    pub(super) fn save_options_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.set_width(full_width(&ui));
            // let w = ui.max_rect().max.x - ui.max_rect().min.x;
            // ui.set_width(w);
            if ui.button("Save as...").clicked() {
                info!("Saving image...");
                if let Some(img) = &self.img {
                    self.save_file_as();
                }
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
                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                    self.output_directory = Some(dir);
                    self.save_into_parent_dir = false;
                }
            }
            ui.checkbox(&mut self.save_into_parent_dir, "Same directory");
        });
        ui.add_space(5.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Saving into: ");
            let text = if self.save_into_parent_dir {
                let mut parent_dir = self.path.as_ref().unwrap().clone();
                parent_dir.pop();
                RichText::new(parent_dir.to_string_lossy()).monospace()
            } else if let Some(output_dir) = &self.output_directory {
                RichText::new(output_dir.to_string_lossy()).monospace()
            } else {
                RichText::new("No output directory set").italics()
            };
            ui.label(text);
        });
    }

    pub(super) fn video_panel(&mut self, ui: &mut Ui) {
        let mut current_frame = None;
        let mut do_export = false;
        if let Some(player) = &mut self.video_player {
            ui.with_layout(Layout::bottom_up(Align::Center).with_cross_justify(true), |ui|{
                ui.group(|ui| {
                    ui.horizontal(|ui|{
                        if ui.button("Save Frame").clicked() {
                            current_frame = player.video_streamer.lock().current_frame().and_then(|frame|{
                                Some(colimg_to_rgbimg(&frame))
                            });
                        }
                        if ui.button("Start Export").clicked() {
                            do_export = true;
                            player.pause();
                        }
                        ui.checkbox(&mut player.options.looping, "Loop");

                        if ui.button("Mute/Unmute").clicked() {
                            if player.options.audio_volume.get() != 0. {
                                player.options.audio_volume.set(0.)
                            } else {
                                player.options
                                    .audio_volume
                                    .set(player.options.max_audio_volume / 2.)
                            }
                        }
                        ui.add_space(ui.available_width());
                    });
                });
                ui.with_layout(Layout::top_down(Align::Min), |ui|{
                    let scale_to_fit = (ui.available_width() / player.size.x).min(ui.available_height() / player.size.y) ;
                    player.ui(ui, player.size * scale_to_fit);
                });
            });
        }

        if let Some(frame) = current_frame {
            save_image_as(&frame, self.path.as_deref());
        }
        if do_export {
            if let Some(path) = self.path.clone() {
                // Choose file
                let file = rfd::FileDialog::new()
                    .add_filter("Videos", &["mp4", "mov", "webm", "avi", "mkv", "m4v", "mpg"])
                    .save_file();
                if let Some(output) = file {
                    // Start sorting the video in the background
                    let ps = self.values.to_pixelsorter();
                    self.video_thread_phone = Some(
                        ps.sort_video_threaded(
                            path.to_string_lossy().as_str(),
                            output.to_string_lossy().as_str()
                        )
                    );
                };
            } else {
                panic!("Video opened, but no path set");
            }
        }
    }

    pub(super) fn layering_panel(&mut self, flex: &mut FlexInstance) {
        if let Some(ls) = &mut self.layered_sorter {
            let mut layer_to_select = None;
            let mut layer_to_delete = None;

            let item_frame = Frame::default()
                .inner_margin(0.0)
                .stroke(Stroke::new(1.0, Color32::DARK_GRAY));

            // ADD LAYER (+)
            let button = Button::new(RichText::new("+").heading());
            if flex.add(item().basis(30.0).grow(0.0), button).clicked() {
                ls.add_layer(ls.get_current_layer().get_sorting_values().clone());
                // self.change_layer = SwitchLayerMessage::Layer(ls.get_layers().len() - 1);
                layer_to_select = Some(ls.get_layers().len() - 1);
            }

            let only_one_layer = ls.get_layers().len() == 1;

            for (i, layer) in ls.get_layers().iter().enumerate().rev() {
                let values = layer.get_sorting_values();

                let layer_name = RichText::new(format!(
                    "[{}] {}",
                    i,
                    layer
                        .get_sorting_values()
                        .to_pixelsorter()
                        .to_pretty_short_string()
                ));

                let button = SelectableLabel::new(
                    ls.get_current_index() == i && !self.show_base_image,
                    layer_name.monospace().size(10.5),
                );

                let delete_button = Button::new("X").corner_radius(0.0);

                // Add a layer button
                flex.add_flex(item().basis(30.0).frame(item_frame), Flex::horizontal(), |flex| {
                    flex.add_ui(item(), |ui|{
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                            let res_del = ui.add_enabled_ui(!only_one_layer, |ui|{
                                ui.add_sized(vec2(20.0, 30.0), delete_button)
                            }).inner;
                            let res = ui.add_sized(ui.available_size(), button);
                            //let res = ui.add_sized(vec2(ui.available_width(), 30.0), button);

                            // Adding and removing on clicks
                            if res.clicked() {
                                layer_to_select = Some(i);
                            }
                            if res.middle_clicked() || res_del.clicked() {
                                layer_to_delete = Some(i);
                            }
                        });
                    });
                });
            }

            // Show-base-image button
            let base_image_button = SelectableLabel::new(
                self.show_base_image,
                RichText::new("[Original Image]").underline().monospace(),
            );
            flex.add_ui(item().basis(30.0).frame(item_frame), |ui| {
                if ui
                    .add_sized(ui.available_size(), base_image_button)
                    .clicked()
                {
                    self.change_layer = SwitchLayerMessage::BaseImage;
                }
            });

            // The loop told us something has to be selected/deleted
            // We set a flag so that is done at the end of the gui update function
            if let Some(i) = layer_to_select {
                self.change_layer = SwitchLayerMessage::Layer(i);
            }
            if let Some(i) = layer_to_delete {
                self.change_layer = SwitchLayerMessage::DeleteLayer(i);
            }
        } else {
            // When there is no layered sorter, f.e. when sorting videos
            let item_frame = Frame::default()
                .inner_margin(0.0)
                .stroke(Stroke::new(1.0, Color32::DARK_GRAY));

            flex.add_ui(item().basis(30.0).frame(item_frame), |ui| {
                ui.set_min_width(ui.available_width());
                ui.add_sized(ui.available_size(), Label::new("[No Layers]"));
            });
        }
    }
}
