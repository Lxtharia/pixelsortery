#![allow(unused)]
use eframe::egui::{
    self, Align, Button, Color32, Image, Key, Layout, Modifiers, RichText, ScrollArea,
    TextureFilter, TextureHandle, TextureOptions, Ui, Vec2,
};
use egui::{ColorImage, Hyperlink, Modal, Rgba, Spinner, scroll_area::ScrollBarVisibility, style::ScrollStyle};
use egui_flex::{item, Flex, FlexAlign, FlexAlignContent, FlexJustify};
#[cfg(feature = "video")]
use egui_video::PlayerState;
use image::{Pixel, Rgb, RgbImage, RgbaImage, EncodableLayout, ImageEncoder, codecs::png::PngEncoder};
use inflections::case::to_title_case;
use layers::LayeredSorter;
use log::{debug, info, warn};
use pixelsortery::{
    CachedPixelsorter, Pixelsorter, path_creator::PathCreator, pixel_selector::{
        PixelSelectCriteria,
        PixelSelector::{self, *},
    }, span_sorter::{SortingAlgorithm, SortingCriteria}
};
#[cfg(feature = "video")]
use pixelsortery::{Progress, ThreadPhone,};
use std::{
    ffi::OsString, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender, channel}}
};
use web_time::{Duration, Instant};
use tokio;


use crate::{AUTHORS, PACKAGE_NAME, VERSION};
mod layers;

/// How long the "Saved" Label should be visible before it vanishes
const SAVED_LABEL_VANISH_TIMEOUT: f32 = 2.0;
const INITIAL_WINDOW_SIZE: Vec2 = egui::vec2(1000.0, 700.0);

mod components;

pub fn init(ps: Option<&Pixelsorter>, img: Option<(RgbImage, PathBuf)>, video: Option<PathBuf>) -> eframe::Result {
    let mut psgui = PixelsorterGui::default();
    #[cfg(feature = "video")] {
        psgui.audio_device = Some(egui_video::AudioDevice::new());
    }

    if let Some(ps) = ps {
        psgui = psgui.with_values(ps);
    }
    #[cfg(not(feature = "video"))]
    if let Some((img, img_path)) = img {
        psgui = psgui.with_image(img, img_path);
    }
    #[cfg(feature = "video")]
    if let Some((img, img_path)) = img {
        psgui = psgui.with_image(img, img_path);
    }
    else if let Some(video_path) = video {
        psgui.path = Some(video_path);
        psgui.do_open_video = true;
    }

    // Create a thread that receives sorting values, then locks the cache with the original image
    // Sorts the image and sends it back to the gui thread
    let (ra, sb) = (psgui.sort_channel.recv_a.take().unwrap(), psgui.sort_channel.send_b.clone());
    let sorter_arc = psgui.cached_sorter.clone();
    let time_arc = psgui.time_last_sort.clone();
    let prog_arc = psgui.in_progress.clone();

    psgui.sort_channel.send_a.send(psgui.values);
    debug!("Initializing Gui...");
    let sort_async = move || {
        debug!("Code inside the thread");
        while let Ok(mut psv) = ra.recv() {
            let mut counter = 1;
            // Get full 
            while let Ok(newer_psv) = ra.try_recv() { counter += 1; psv = newer_psv }
            info!("[THREAD] Received {counter} requests. Sorting the last one...");
            if let Ok(Some(ps)) = sorter_arc.lock().as_deref_mut() {
                let timestart = Instant::now();
                *prog_arc.lock().unwrap() = true;
                let sorted = ps.sort(&psv.to_pixelsorter());
                *time_arc.lock().unwrap() = timestart.elapsed();
                sb.send(sorted);
                *prog_arc.lock().unwrap() = false;
            }
        }
    };

    // When compiling natively:
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(INITIAL_WINDOW_SIZE),
            ..Default::default()
        };
        eframe::run_native(
            "Pixelsortery",
            options,
            Box::new(|cc| {
                // This gives us image support
                egui_extras::install_image_loaders(&cc.egui_ctx);
                Ok(Box::new(psgui))
            }),
        )?
    }
    // When compiling to web using trunk:
    #[cfg(target_arch = "wasm32")]
    {

        use eframe::wasm_bindgen::JsCast as _;

        // Redirect `log` message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        let web_options = eframe::WebOptions::default();

        rayon::spawn(|| {panic!("If this gets executed, it's good.")});

        rayon::spawn(sort_async);

        wasm_bindgen_futures::spawn_local(async {
            debug!("Started sorting thread");
            let document = web_sys::window()
                .expect("No window")
                .document()
                .expect("No document");

            let canvas = document
                .get_element_by_id("egui-canvas")
                .expect("Failed to find egui-canvas html element")
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .expect("egui-canvas was not a HtmlCanvasElement");

            let start_result = eframe::WebRunner::new()
                .start(
                    canvas,
                    web_options,
                    Box::new(|cc| {
                        // This gives us image support
                        egui_extras::install_image_loaders(&cc.egui_ctx);
                        Ok(Box::new(psgui))
                    }),
                )
            .await;
        });
    }
    Ok(())
}

struct TwoWayChannel<A, B> {
    send_a: Sender<A>,
    recv_a: Option<Receiver<A>>, // Option until msmc is stable
    send_b: Sender<B>,
    recv_b: Receiver<B>,
}

/// Struct holding all the states of the gui and values of sliders etc.
struct PixelsorterGui {
    /// The path of the loaded image
    path: Option<PathBuf>,
    /// The Values and sorted Images, layered
    layered_sorter: Option<LayeredSorter>,
    /// Sorter with improved performance
    cached_sorter: Arc<Mutex<Option<CachedPixelsorter>>>,
    /// Channels to communicate to the sorting thread
    sort_channel: TwoWayChannel<PixelsorterValues, RgbImage>,
    in_progress: Arc<Mutex<bool>>,
    /// All the adjustable values for the pixelsorter
    values: PixelsorterValues,
    show_mask: bool,
    /// The current image from the selected layer
    img: Option<RgbImage>,
    /// The image used by egui to draw every frame
    texture: Option<TextureHandle>,
    output_directory: Option<PathBuf>,
    save_into_parent_dir: bool,
    time_last_sort: Arc<Mutex<Duration>>,
    auto_sort: bool,
    do_sort: bool,
    saving_success_timeout: Option<Instant>,
    change_layer: SwitchLayerMessage,
    show_base_image: bool,
    #[cfg(feature = "video")]
    audio_device: Option<egui_video::AudioDevice>,
    #[cfg(feature = "video")]
    video_player: Option<egui_video::Player>,
    #[cfg(feature = "video")]
    do_open_video: bool,
    /// A tuple for communicating with the current sorting thread
    #[cfg(feature = "video")]
    video_thread_phone: Option<ThreadPhone>,
    img_tx: Sender<RgbImage>,
    img_rx: Receiver<RgbImage>,
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
        let (send_a, recv_a) = channel();
        let (send_b, recv_b) = channel();
        let (tx, rx)         = channel();
        Self {
            path: None,
            layered_sorter: None,
            cached_sorter: Arc::new(Mutex::new(None)),
            img: None,
            texture: None,
            show_mask: false,
            sort_channel: TwoWayChannel { send_a, recv_a: Some(recv_a), send_b, recv_b },
            in_progress: Arc::new(Mutex::new(false)),
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
            time_last_sort: Arc::new(Mutex::new(Duration::default())),
            auto_sort: true,
            output_directory: None,
            save_into_parent_dir: false,
            saving_success_timeout: None,
            change_layer: SwitchLayerMessage::None,
            do_sort: true,
            show_base_image: false,
            #[cfg(feature = "video")]
            audio_device: None,
            #[cfg(feature = "video")]
            video_player: None,
            #[cfg(feature = "video")]
            do_open_video: false,
            #[cfg(feature = "video")]
            video_thread_phone: None,
            img_tx: tx,
            img_rx: rx,
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
        *self.cached_sorter.lock().unwrap() = Some(CachedPixelsorter::new(self.img.clone().unwrap()));
        self.path = Some(image_path);
        self
    }

    /// Calls sort_current_layer, sets the image and texture
    fn sort_img(&mut self, ctx: &egui::Context, force: bool) {
        let ps = self.values.to_pixelsorter();
        if self.cached_sorter.lock().unwrap().is_none() && self.img.is_some() {
            *self.cached_sorter.lock().unwrap() = Some(CachedPixelsorter::new(self.img.clone().unwrap()));
        }
        if let Ok(Some(cached)) = self.cached_sorter.lock().as_deref_mut() {
            let timestart = Instant::now();
            self.img = Some(cached.sort(&ps));
            *self.time_last_sort.lock().unwrap() = timestart.elapsed();
        }

        // TOOD: Make layered sorter use cache
        self.layered_sorter = None;

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
                    *self.time_last_sort.lock().unwrap() = timestart.elapsed();
                }
                self.img = Some(ls.get_current_layer().get_img().clone());
            }
        }
        // Display sorted image
        self.update_texture(ctx);
    }

    fn queue_sort(&self, force: bool) {
        // Add to queue
        info!("[UI] Sending values to thread!");
        self.sort_channel.send_a.send(self.values);
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
            let sized_img = Image::new((tex.id(), tex.size_vec2())).shrink_to_fit();
            let rect = egui::Rect {
                min: ui.cursor().min,
                max: ui.cursor().min + sized_img.calc_size(ui.available_size(), Some(tex.size_vec2())),
            };
            egui::Frame::group(ui.style_mut())
                .inner_margin(0)
                .show(ui, |ui| {
                    ui.add(sized_img);
                });
            // TODO: show only after a small timeout
            if self.in_progress.lock().is_ok_and(|it| *it) {
                ui.put(rect, Spinner::new());
            }
        }
    }

    fn open_file_dialog(&mut self, ctx: &egui::Context) -> () {
        // Opening image until cancled or until valid image loaded
        #[cfg(not(target_arch = "wasm32"))]
        loop {
            let mut filedialog = rfd::FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"]);
            #[cfg(feature = "video")]
            filedialog.add_filter("Images and Videos", &["png", "jpg", "jpeg", "webp", "mp4", "mov", "webm", "avi", "mkv", "m4v", "mpg"])
                .add_filter("Videos", &["mp4", "mov", "webm", "avi", "mkv", "m4v", "mpg"]);
            let file = filedialog.pick_file();
            match file {
                None => break,
                Some(f) => match image::open(f.as_path()) {
                    Ok(i) => {
                        let img = i.into_rgb8();
                        if let Some(ls) = &mut self.layered_sorter {
                            ls.set_base_img(img.clone());
                        }
                        // I want to make layered_sorter mandatory and remove the possibility of it being None
                        #[cfg(feature = "video")] {
                            self.video_player = None;
                        }
                        self.img = Some(img);
                        *self.cached_sorter.lock().unwrap() = Some(CachedPixelsorter::new(self.img.clone().unwrap()));
                        self.update_texture(ctx);
                        self.path = Some(f);
                        break;
                    }
                    Err(_) => {
                        #[cfg(feature = "video")]
                        if self.init_video(ctx, &f).is_ok() {
                            // TODO: Either use layered sorter or save it for the next loaded image
                            self.layered_sorter = None;
                            self.img = None;
                            self.path = Some(f);
                            break;
                        }
                    }
                },
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let i_sx = self.img_tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                .await;

                if let Some(file) = file {
                    // If you are on native platform you can just get the path
                    #[cfg(not(target_arch = "wasm32"))]
                    println!("{:?}", file.path());

                    // If you care about wasm support you just read() the file
                    let f = file.read().await;
                    match image::load_from_memory(&file.read().await) {
                        Ok(i) => {
                            let img = i.into_rgb8();
                            i_sx.send(img).expect("Couldnt send image x.x");
                        }
                        Err(_) => {}
                    }

                }
            });
        }
    }

    /// Sorts and saves the image to the current output directory with a given filename
    fn save_file_to_out_dir(&mut self) -> () {
        if let Some(path) = &self.path {
            let (basename, ext) = (
                path.file_stem()
                    .map(|s| s.to_os_string())
                    .unwrap_or(OsString::from("img")),
                path.extension()
                    .map(|s| s.to_os_string())
                    .unwrap_or(OsString::from("png")),
            );

            let mut ps = &self.values.to_pixelsorter();
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

    /// Sorts and saves the current image to a location via a file picker
    fn save_file_as(&mut self) {
        let path = &self.path.clone();
        if let Some(img) = &self.img {
            if save_image_as(img, path.as_deref()).is_ok(){
                self.saving_success_timeout = Some(Instant::now());
            };
        }
    }

    #[cfg(feature = "video")]
    fn init_video(&mut self, ctx: &egui::Context, video_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        /* called once (creating a player) */

        let mut player = egui_video::Player::new(ctx, &video_path.to_string_lossy().to_string())
            .and_then(|player|{
                if let Some(audio) = &mut self.audio_device {
                    player.with_audio(audio)
                } else {
                    Ok(player)
                }
            });
        self.video_player = Some(player?);
        if let Some(player) = &mut self.video_player {
            player.options.looping = false;
            self.do_sort = true;
        }
        Ok(())
    }

    #[cfg(feature = "video")]
    fn video_mode(&self) -> bool {
        self.img.is_none() && self.video_player.is_some()
    }

}

impl eframe::App for PixelsorterGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut do_open_file = false;
        // Shortcuts
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::S) {
                if let Some(img) = &self.img {
                    self.save_file_as();
                }
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
            // style.debug.debug_on_hover_with_all_modifiers = true;
        });


        #[cfg(target_arch = "wasm32")] // Receive image from file picker
        if let Ok(img) = self.img_rx.try_recv() {
            if let Some(ls) = &mut self.layered_sorter {
                ls.set_base_img(img.clone());
            }
            self.img = Some(img);
            self.update_texture(ctx);
        }
        // Open file on startup
        if do_open_file {
            self.open_file_dialog(ctx);
        }

        // Load video if no image is set
        #[cfg(feature = "video")]
        if self.do_open_video {
            if let Some(video_path) = &self.path.clone() {
                self.init_video(ctx, video_path);
            }
            self.do_open_video = false;
        }

        // Receive sorted image
        if let Ok(sorted_img) = self.sort_channel.recv_b.try_recv() {
            self.img = Some(sorted_img);
            self.update_texture(ctx);
        }

        // stuff
        if let Some(ls) = &self.layered_sorter {
            // Load current values
            self.values = ls.get_current_layer().get_sorting_values().clone();
        } else {
            // Create a layering thingy if we don't have one yet
            if let Some(img) = &self.img {
                self.layered_sorter = Some(LayeredSorter::new(img.clone(), self.values));
            }
        }

        // UI //

        #[cfg(feature = "video")]
        // Show modal if a video is currently being exported
        if let Some(phone) = &mut self.video_thread_phone {
            if phone.join_handle.is_finished() {
                self.video_thread_phone = None;
            } else {
                ctx.request_repaint();
                Modal::new("VideoInProgressModal".into())
                    .show(ctx, |ui|{
                        // Receive all progress updates and save the most recent one
                        while let Ok(prog) = phone.progress_receiver.try_recv() {
                            phone.last_progress = prog;
                        };
                        ui.style_mut().spacing.window_margin *= 2.0;
                        ui.heading("Sorting in progress...");
                        ui.vertical_centered(|ui| {
                            ui.set_width(300.0);
                            ui.separator();
                            ui.vertical(|ui|{
                                ui.label(format!("Frames sorted:\t {:<5}",  phone.last_progress.current_frame));
                                ui.label(format!("Time elapsed:\t {: <20?}", phone.last_progress.elapsed_time));
                            });
                            ui.add_space(5.0);
                            let cancel_button = Button::new(RichText::new("Stop").heading())
                                .selected(phone.cancel_signal.load(Ordering::Relaxed));
                            if ui.add(cancel_button).clicked() {
                                // Send exit "signal" to thread
                                phone.cancel_signal.store(true, Ordering::Relaxed);
                            }
                            ui.add_space(5.0);
                            ui.label(RichText::new("The destination file will not be deleted").small());
                        });
                    });
            }
        }

        egui::TopBottomPanel::bottom("info-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tls = self.time_last_sort.lock().unwrap();
                ui.label(format!("Time of last sort:\t{:?}", tls));
                ui.label(format!(
                    "({:.3} fps)",
                    (1.0 / tls.as_secs_f32())
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
                    ui.horizontal(|ui|{
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(format!(
                            " v{} by {}",
                            VERSION,
                            AUTHORS
                        ));
                        ui.label(" ");
                        ui.hyperlink_to(to_title_case(PACKAGE_NAME), "https://github.com/Lxtharia/pixelsortery");
                    });
                    ui.separator();
                    if let Some(tex) = &self.texture {
                        let [w, h] = tex.size();
                        ui.label(format!("{} x {} ({} pixels)", w, h, w * h));
                        ui.separator();
                    }
                    #[cfg(feature = "video")]
                    if let Some(player) = &self.video_player {
                        let (fps, (w, h)) = (player.framerate, player.size.into());
                        ui.label(format!("{fps} fps | {} x {} ({} pixels)", w, h, w * h));
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
                                self.open_file_dialog(ctx);
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
                                ui.add_enabled_ui(self.layered_sorter.is_some(), |ui|{
                                    ScrollArea::vertical()
                                        // .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                                        .id_salt("LayerScrollArea")
                                        .show(ui, |ui| {
                                            Flex::vertical()
                                                .h_full()
                                                .w_full()
                                                .wrap(false)
                                                .justify(FlexJustify::End)
                                                .align_items(FlexAlign::Stretch)
                                                .show(ui, |flex| {
                                                    // flex.grow();
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
            self.queue_sort(true);
            #[cfg(feature = "video")]
            // Update the video filter function to use the new values
            if let Some(player) = &mut self.video_player {
                let sorter = self.values.to_pixelsorter();
                let time_last_sort_arc = self.time_last_sort.clone();
                player.video_streamer.lock().filter_video_frame_fn = Some(create_frame_filter(sorter, time_last_sort_arc));
                if player.player_state.get() != PlayerState::Playing {
                    let current_frame = player.video_streamer.lock().current_frame();
                    if let Some(current_frame) = current_frame {
                        player.set_current_frame(current_frame);
                    }
                }
            }
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
        #[cfg(not(feature = "video"))]
        egui::CentralPanel::default().show(ctx, |ui| {
                self.show_img(ui);
        });
        #[cfg(feature = "video")]
        // Display the image or video!
        egui::CentralPanel::default().show(ctx, |ui| {
            use egui_video;
            if self.video_player.is_some() {
                self.video_panel(ui);
            } else {
                self.show_img(ui);
            }
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

/// opens a file dialog to let the user choose a destination for a file
/// an optional path can be provided that is used to suggest a new filename
pub fn save_image_as(img: &RgbImage, original_path: Option<&Path>) -> Result<(), image::ImageError> {
    let mut suggested_filename = String::from("");
    if let Some(p) = original_path {
        if let (Some(stem), Some(ext)) = (p.file_stem(), p.extension()) {
            suggested_filename = format!(
                "{}-sorted.{}",
                stem.to_string_lossy(),
                ext.to_string_lossy()
            );
        };
    };
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .set_file_name(&suggested_filename)
            .save_file();
        if let Some(f) = file {
            let res = img.save(&f);
            if let Err(err_msg) = &res {
                warn!(
                    "Saving image to {} failed: {}",
                    f.to_string_lossy(),
                    err_msg
                );
            } else {
                info!("Saved file to '{}' ...", f.to_string_lossy());
            };
            res
        } else {
            Ok(())
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        use base64::{Engine as _, engine::{self, general_purpose}};
        use std::io::Cursor;
        use web_sys;
        use eframe::wasm_bindgen::JsCast;

        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();

        let mut bytes: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png).unwrap();

        let link = doc.create_element("a").unwrap();
        link.set_attribute("href", format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(bytes)).as_str());
        link.set_attribute("download", "sorted.png");

        let link: web_sys::HtmlAnchorElement = web_sys::HtmlAnchorElement::unchecked_from_js(link.into());
        link.click();
        Ok(())
    }
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

pub fn create_frame_filter(sorter: Pixelsorter, time_last_sort_arc: Arc<Mutex<Duration>> )
    -> Box<dyn FnMut(&mut ColorImage) + Send>
{
    Box::new(move |frame| {
        let timer = Instant::now();
        let mut img = colimg_to_rgbaimg(frame);
        let (w, h) = (img.width().into(), img.height().into());
        let pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().map(|p|{
            Rgb::from_slice_mut(&mut p.0[0..3])
        }).collect();
        sorter.sort_pixels(pixels, w, h);
        let newimg = ColorImage::from_rgba_unmultiplied(frame.size, img.as_raw());
        *frame = newimg;
        *time_last_sort_arc.lock().unwrap() = timer.elapsed();
    })
}

fn colimg_to_rgbimg(cimg: &ColorImage) -> RgbImage  {
    let (w, h) = (cimg.width(), cimg.height());
    let mut buffer = Vec::with_capacity(w*h*3);
    for c in cimg.pixels.iter() {
        buffer.push(c.r());
        buffer.push(c.g());
        buffer.push(c.b());
    }
    RgbImage::from_raw(w as u32, h as u32, buffer).unwrap()
}

fn colimg_to_rgbaimg(cimg: &ColorImage) -> RgbaImage  {
    let (w, h) = (cimg.width(), cimg.height());
    let mut buffer = Vec::with_capacity(w*h*3);
    for c in cimg.pixels.iter() {
        buffer.push(c.r());
        buffer.push(c.g());
        buffer.push(c.b());
        buffer.push(c.a());
    }
    RgbaImage::from_raw(w as u32, h as u32, buffer).unwrap()
}
