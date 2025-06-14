#![allow(unused_parens, unused)]
use eframe::egui::TextBuffer;
use image::{codecs::png::PngEncoder, GenericImageView, ImageResult, Rgb, RgbImage};
use log::{debug, error, info, warn};
use path_creator::PathCreator;
use rayon::prelude::*;
use span_sorter::{SortingCriteria, SpanSorter};
use std::{fmt::Debug, fs, io::{ErrorKind, Read, Write}, path::{Path, PathBuf}, process::{self, Command, Output, Stdio}, time::Instant};
use ffmpeg_the_third::{self as ffmpeg, codec::{self, Parameters}, frame, media, packet, software::{scaler, scaling}, Stream};

use crate::pixel_selector::PixelSelector;

mod color_helpers;
pub mod path_creator;
pub mod pixel_selector;
pub mod span_sorter;

#[derive(Clone)]
pub struct Pixelsorter {
    pub sorter: span_sorter::SpanSorter,
    pub selector: PixelSelector,
    pub path_creator: path_creator::PathCreator,
    pub reverse: bool,
}

pub type Span = Vec<Rgb<u8>>;

impl Pixelsorter {
    // constructor
    pub fn new() -> Pixelsorter {
        Pixelsorter {
            sorter: SpanSorter::new(SortingCriteria::Brightness),
            selector: PixelSelector::Full,
            path_creator: PathCreator::AllHorizontally,
            reverse: false,
        }
    }
    pub fn to_long_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "All Horizontally".into(),
            PathCreator::AllVertically => "All Vertically".into(),
            PathCreator::HorizontalLines => if self.reverse {"Left"} else {"Right"}.into(),
            PathCreator::VerticalLines => if self.reverse {"Up"} else {"Down"}.into(),
            PathCreator::Circles => "Circles".into(),
            PathCreator::Spiral => "Spiral".into(),
            PathCreator::SquareSpiral => "Square Spiral".into(),
            PathCreator::RectSpiral => "Rect Spiral".into(),
            PathCreator::Diagonally(a) => format!("Diagonally ({}°)", a),
            PathCreator::Hilbert => "Hilbert Curve".into(),
            p => format!("{}", p),
        }
        .as_str();
        s += "-";
        if self.reverse {
            s += "R-"
        };
        s += match self.selector {
            PixelSelector::Full => "Full".into(),
            PixelSelector::Fixed { len } => format!("Fixed length ({})", len),
            PixelSelector::Random { max } => format!("Random length ({})", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{}{}-{}",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "Hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "Brightness",
                    pixel_selector::PixelSelectCriteria::Saturation => "Saturation",
                },
                min,
                max
            ),
        }
        .as_str();
        s += "-";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "Mapsort",
            span_sorter::SortingAlgorithm::Shellsort => "Shellsort",
            span_sorter::SortingAlgorithm::Glitchsort => "Glitchsort",
            span_sorter::SortingAlgorithm::DebugColor => "Debug-colors",
        };
        s += "-";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "Hue",
            SortingCriteria::Brightness => "Brightness",
            SortingCriteria::Saturation => "Saturation",
        };

        s
    }

    pub fn to_pretty_short_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "Horizontal".into(),
            PathCreator::AllVertically => "Vertical".into(),
            PathCreator::HorizontalLines => if self.reverse {"Left"} else {"Right"}.into(),
            PathCreator::VerticalLines => if self.reverse {"Up"} else {"Down"}.into(),
            PathCreator::Circles => "Circles".into(),
            PathCreator::Spiral => "Spiral".into(),
            PathCreator::SquareSpiral => "Square".into(),
            PathCreator::RectSpiral => "Rect".into(),
            PathCreator::Diagonally(a) => format!("Diag({}°)", a),
            PathCreator::Hilbert => "Hilbert".into(),
            p => format!("{}", p),
        }
        .as_str();
        if self.reverse {
            s += "{R}"
        };
        s += " | ";

        s += match self.selector {
            PixelSelector::Full => "Full".into(),
            PixelSelector::Fixed { len } => format!("Fixed ({})", len),
            PixelSelector::Random { max } => format!("Random ({})", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{} ({}-{})",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "Hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "Bright",
                    pixel_selector::PixelSelectCriteria::Saturation => "Sat",
                },
                min,
                max
            ),
        }
        .as_str();
        s += " | ";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "Map",
            span_sorter::SortingAlgorithm::Shellsort => "Shell",
            span_sorter::SortingAlgorithm::Glitchsort => "Glitch",
            span_sorter::SortingAlgorithm::DebugColor => "Debug",
        };
        s += "(by ";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "Hue",
            SortingCriteria::Brightness => "Bright",
            SortingCriteria::Saturation => "Sat",
        };
        s += ")";

        s
    }

    pub fn to_compact_string(&self) -> String {
        let mut s = String::new();
        s += match self.path_creator {
            PathCreator::AllHorizontally => "hor".into(),
            PathCreator::AllVertically => "vert".into(),
            PathCreator::HorizontalLines => "lr".into(),
            PathCreator::VerticalLines => "ud".into(),
            PathCreator::Circles => "circ".into(),
            PathCreator::Spiral => "sprl".into(),
            PathCreator::SquareSpiral => "spSq".into(),
            PathCreator::RectSpiral => "spRe".into(),
            PathCreator::Diagonally(a) => format!("diag{}", a),
            PathCreator::Hilbert => "hilb".into(),
            p => format!("{}", p).to_lowercase(),
        }
        .as_str();
        s += "-";
        if self.reverse {
            s += "R-"
        };
        s += match self.selector {
            PixelSelector::Full => "full".into(),
            PixelSelector::Fixed { len } => format!("fixed{}", len),
            PixelSelector::Random { max } => format!("rand{}", max),
            PixelSelector::Threshold { min, max, criteria } => format!(
                "{}{}-{}",
                match criteria {
                    pixel_selector::PixelSelectCriteria::Hue => "hue",
                    pixel_selector::PixelSelectCriteria::Brightness => "bright",
                    pixel_selector::PixelSelectCriteria::Saturation => "sat",
                },
                min,
                max
            ),
        }
        .as_str();
        s += "-";
        s += match self.sorter.algorithm {
            span_sorter::SortingAlgorithm::Mapsort => "map",
            span_sorter::SortingAlgorithm::Shellsort => "shell",
            span_sorter::SortingAlgorithm::Glitchsort => "gl",
            span_sorter::SortingAlgorithm::DebugColor => "debug",
        };
        s += "-";
        s += match self.sorter.criteria {
            SortingCriteria::Hue => "hue",
            SortingCriteria::Brightness => "bright",
            SortingCriteria::Saturation => "sat",
        };

        s
    }

    // sorting without creating spans
    pub fn sort_all_pixels(&self, img: &mut RgbImage) {
        let mut pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        self.sorter.sort(&mut pixels);
    }

    /// Sort a given image in place
    pub fn sort(&self, img: &mut RgbImage) {
        let mut timestart = Instant::now();
        // a vector containing pointers to each pixel
        let pixelcount = img.width() * img.height();
        info!(
            "Image information: {} x {} ({} pixels)",
            img.width(),
            img.height(),
            pixelcount
        );

        info!(
            "Sorting with:\n   | {}{}\n   | {}\n   | {}",
            self.path_creator.info_string(),
            if self.reverse { " [Reversed]" } else { "" },
            self.selector.info_string(),
            self.sorter.info_string(),
        );

        timestart = Instant::now();
        // CUT IMAGE INTO PATHS
        let ranges = self.path_creator.create_paths(img, self.reverse);

        info!("TIME [Creating Paths]:\t{:?}", timestart.elapsed());
        timestart = Instant::now();

        // CREATE SPANS ON EVERY PATH
        let mut spans: Vec<Vec<&mut Rgb<u8>>> = Vec::new();
        spans.par_extend(
            ranges
                .into_par_iter()
                .map(|r| self.selector.create_spans(&mut r.into()))
                .flatten(),
        );

        info!("TIME [Selector]:\t{:?}", timestart.elapsed());

        info!("Amount of spans:\t{}", &spans.len());
        timestart = Instant::now();

        // SORT EVERY SPAN
        spans.into_par_iter().for_each(|mut span| {
            self.sorter.sort(&mut span);
        });

        let timeend = timestart.elapsed();
        info!("TIME [Sorting]: \t{:?}", timeend);
    }

    /// Reads a video stream from a file, sorts every frame and then writes it to another file
    /// Hacky, but hopefully better/faster than my shitty bash script
    pub fn sort_video(&self, input: &str, output: &str) {
        // TODO: Create mkfifos
        // Will only work on linux though :sob:
        match self.try_sort_video2(input, output) {
            Ok(_) => println!("Success!"),
            Err(e) => error!("Error sorting video: {e}")
        };
    }

    fn try_sort_video2(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        ffmpeg::init()?;
        // Copied from https://github.com/shssoichiro/ffmpeg-the-third/blob/master/examples/dump-frames.rs

        let mut ictx = ffmpeg::format::input(input_path)?;
        let mut octx = ffmpeg::format::output(output_path)?;


        info!("[VIDEO] Opening input file and decoder");

        // Print information about the input file
        ffmpeg::format::context::input::dump(&ictx, 0, Some(&input_path));
        // -------- Open input stream 
        let input_stream = ictx.streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input_stream.index(); // The stream index we want to manipulate

        // Create a corresponding output stream for each input stream
        for (idx, stream) in ictx.streams().enumerate() {
            // TODO: copy code
            // if idx == video_stream_index { continue; }
            // let mut new_stream = octx.add_stream(codec).unwrap();
            // new_stream.set_parameters(stream.parameters());
            // new_stream.set_time_base(stream.time_base());
        }
        // Guess the codec from output format and add a stream for that
        let mut codec = ffmpeg::encoder::find(octx.format().codec(output_path, media::Type::Video)).unwrap();

            // Stuff
            let mut context = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
            // Boost performance, hell yeah!
            if let Ok(parallelism) = std::thread::available_parallelism() {
                context.set_threading(ffmpeg::threading::Config {
                    kind: ffmpeg::threading::Type::Frame,
                    count: parallelism.get(),
                });
            }
        // A decoder. `send_packet()` to it and `receive_frame()` from it
        let mut decoder = context.decoder().video()?;


        // -------- Create encoder and output_stream

        const FORCED_TB: ffmpeg::Rational = ffmpeg::Rational(1, 30);
        let mut opts = ffmpeg::Dictionary::new();
        opts.set("preset", "medium");
        // Add Encoder with specific codec
        info!("[VIDEO] Opening encoder?");
        let mut prep_encoder = ffmpeg::codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()?;
        // Set a bunch of optins, including preset=medium
        // output_stream.set_parameters(Parameters::from(&prep_encoder));
        prep_encoder.set_width(decoder.width());
        prep_encoder.set_height(decoder.height());
        prep_encoder.set_aspect_ratio(decoder.aspect_ratio());
        prep_encoder.set_format(decoder.format());
        prep_encoder.set_frame_rate(decoder.frame_rate());
        prep_encoder.set_time_base(FORCED_TB);
        // "Open" encoder, whatever that means
        let mut encoder = prep_encoder .open_with(opts) .expect("Error opening encoder with supplied settings");

        // Set parameters and extract some data
        let mut output_stream = octx.add_stream(codec)?;
        output_stream.set_parameters(Parameters::from(&encoder));
        output_stream.set_time_base(FORCED_TB);
        output_stream.set_rate(FORCED_TB); // Just for metadata
        let ist_time_base = input_stream.time_base();
        let ost_time_base = output_stream.time_base();
        let nb_frames = input_stream.frames();
        info!("[VIDEO] Stage 1 complete: V_index: {video_stream_index} | ist_TB: {ist_time_base} | ost_TB: {ost_time_base}");

        //
        // Preperation done (?)
        //

        octx.set_metadata(ictx.metadata().to_owned());
        ffmpeg::format::context::output::dump(&octx, 0, Some(&output_path));
        octx.write_header().unwrap();

        let (width, height, format) = (decoder.width(), decoder.height(), decoder.format());
        // No idea. Somehow important to get good image data
        let mut yuv_to_rgb = scaling::context::Context::get(
            format, width, height,
            ffmpeg::format::Pixel::RGB24, width, height,
            scaling::Flags::BILINEAR,
        )?;
        let mut rgb_to_yuv = scaling::context::Context::get(
            ffmpeg::format::Pixel::RGB24, width, height,
            format, width, height,
            scaling::Flags::BILINEAR,
        )?;

        let mut frame_count = 0;

        let mut manipulate_frame = |in_frame: &mut frame::Video| -> frame::Video {
            let mut rgb_frame = frame::Video::empty();
            yuv_to_rgb.run(in_frame, &mut rgb_frame);

            // Processing, yippie!
            info!("\tTrying to decode frame: {:?}, {}", rgb_frame.width(), rgb_frame.planes());
            let mut img: RgbImage = RgbImage::from_raw(
                rgb_frame.width(),
                rgb_frame.height(),
                rgb_frame.data(0).to_vec()
            ).unwrap();
            info!("\tDecoded Frame {}x{}", img.width(), img.height());
            let mut yuv_frame = frame::Video::empty();

            rgb_to_yuv.run(&rgb_frame, &mut yuv_frame); // TODO: reverse scale??
            info!("\tEncoded to: {:?}, {}", yuv_frame.width(), yuv_frame.planes());
            yuv_frame
        };

        let mut receive_and_process_encoded_frames =
            |
            encoder: &mut ffmpeg::encoder::video::Video,
            out: &mut ffmpeg::format::context::Output,
            | -> Result<(), ffmpeg::Error> {
                // Receive encoded frame
                let mut encoded_packet = ffmpeg::Packet::empty();
                while encoder.receive_packet(&mut encoded_packet).is_ok() {
                    let oldts = encoded_packet.pts();
                    encoded_packet.set_stream(video_stream_index);
                    // encoded_packet.rescale_ts(ist_time_base, ost_time_base);
                    println!("\t OLD TS: {:?} | {} -> {} | NEW TS: {:?}", oldts, ist_time_base, ost_time_base, encoded_packet.pts());
                    // println!("TS: {}", encoded_packet.timestamp());
                    encoded_packet.write_interleaved(out).unwrap();
                }
                Ok(())
            };
        // Will be called after every packet sent
        let mut receive_and_process_decoded_frames =
            |
            decoder: &mut ffmpeg::decoder::Video,
            encoder: &mut ffmpeg::encoder::video::Video,
            out: &mut ffmpeg::format::context::Output,
            | -> Result<(), ffmpeg::Error>
            {
                let mut decoded_frame = frame::Video::empty();
                while decoder.receive_frame(&mut decoded_frame).is_ok() {
                    frame_count += 1;
                    let timestamp = decoded_frame.timestamp();
                    println!("Processing frame [{:>5} / {}] | {timestamp:?}", frame_count, nb_frames);

                    // let mut sorted_frame = manipulate_frame(&mut decoded_frame);
                    let mut sorted_frame = decoded_frame.clone();

                    // sorted_frame.set_pts(timestamp.or(Some(frame_count)));
                    // sorted_frame.set_kind(ffmpeg::picture::Type::None);

                    // And back into the encoder it goes
                    encoder.send_frame(&sorted_frame).unwrap();

                    receive_and_process_encoded_frames(encoder, out);
                }

                Ok(())
            };


        // Go through each data packet and do stuff
        for (stream, mut packet) in ictx.packets().filter_map(Result::ok) {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet).unwrap();
                receive_and_process_decoded_frames(&mut decoder, &mut encoder, &mut octx)?;
            } else {
                // TODO: Clone every other stream
                /*
                   packet.set_stream(stream.index());
                // No idea
                packet.rescale_ts(
                stream.time_base(),
                octx.stream(stream.index()).unwrap().time_base()
                );
                // Copy any other stream packets
                packet.write_interleaved(&mut octx);
                */
            }
        }

        // Flush encoders and decoders.
        decoder.send_eof().unwrap();
        receive_and_process_decoded_frames(&mut decoder, &mut encoder, &mut octx)?;
        encoder.send_eof().unwrap();
        receive_and_process_encoded_frames(&mut encoder, &mut octx);
        octx.write_trailer().unwrap();

        Ok(())
    }

    fn try_sort_video(&self, input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Get video width,height,bytes
        // ffprobe -v error -select_stream v:0 -show_entries stream=width,height
        let cmd_output: Output = Command::new("ffprobe")
            .arg("-v").arg("error")
            .arg("-select_streams").arg("v:0")
            .arg("-count_packets")
            .arg("-show_entries").arg("stream=width,height,nb_read_packets,r_frame_rate")
            .arg("-of").arg("csv=p=0:s=x")
            .arg(input)
            .output().expect("Could not run ffprobe command.");
        let s = String::from_utf8(cmd_output.stdout)
            .expect("Command output was not utf8. Handling this should be fixed tbh.");
        info!("[VIDEO] FFProbe Output: {s}");
        // Most horrible parsing of all time
        let mut words = s.trim().split('x');
        let w: u32 = words.next().expect("Could not parse ffprobe output").parse()?;
        let h: u32 = words.next().expect("Could not parse ffprobe output").parse()?;
        let bytes: u32 = w*h*3;
        let mut frame_rate = words.next().expect("Could not parse ffprobe output").split('/');
        let packet_count: u32 = words.next().expect("Could not parse ffprobe output").parse()?;
        let fps: f64 = frame_rate.next()
            .expect("Could not parse ffprobe output: frame rate")
            .parse::<u32>()? as f64
            /
            frame_rate.next()
            .unwrap_or("1")
            .parse::<u32>()? as f64;

        println!("[VIDEO] Video information: {w}x{h} (= {bytes} bytes/frame) | {fps} fps ");

        // ffmpeg -y -loglevel error -i "$VID" -pix_fmt rgb24 -f rawvideo "$RAW_IN" &
        let mut ff_in = Command::new("ffmpeg")
            .arg("-y")
            .arg("-loglevel").arg("quiet")
            .arg("-i").arg(input)
            .arg("-fps_mode").arg("passthrough")
            .arg("-pix_fmt").arg("rgb24")
            .arg("-f").arg("rawvideo")
            .arg("-")
            .stdout(Stdio::piped())
            .spawn().expect("Failed to run ffmpeg")
            ;

        let mut ff_out = if output == "-" {
            Command::new("ffplay")
                .arg("-loglevel").arg("error")
                .arg("-pixel_format").arg("rgb24")
                .arg("-f").arg("rawvideo")
                .arg("-video_size").arg(format!("{w}x{h}"))
                .arg("-i").arg("-")
                .stdin(Stdio::piped())
                .spawn().expect("Failed to run ffplay")
        } else {
            info!("[VIDEO] Trying to write video to {output}");
            Command::new("ffmpeg")
                .arg("-y")
                .arg("-loglevel").arg("quiet")
                .arg("-f").arg("rawvideo")
                .arg("-pix_fmt").arg("rgb24")
                .arg("-video_size").arg(format!("{w}x{h}"))
                .arg("-i").arg("-")
                .arg("-i").arg(input)
                .arg("-map").arg("0:v")
                .arg("-map").arg("1:a")
                .arg("-c:v").arg("libx264")
                .arg("-pix_fmt").arg("yuv420p")
                .arg("-c:a").arg("copy")
                .arg("-r").arg(fps.to_string())
                .arg(output)
                .stdin(Stdio::piped())
                .spawn().expect("Failed to run ffmpeg")
        };

        // Read stdoutput from in_pipe, sort it and write it to out_pipe
        let mut in_pipe = ff_in.stdout.take().expect("");
        let mut out_pipe = ff_out.stdin.take().expect("");
        let mut buffer = vec![0u8; bytes as usize];
        let mut frame_counter = 1;
        loop {
            // Read exactly that amount of bytes that make one frame
            println!("[VIDEO] Reading Frame [{frame_counter:_>5} / {packet_count}]");
            let timestart = Instant::now();
            match in_pipe.read_exact(&mut buffer) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        info!("[VIDEO] Encountered EOF");
                        break;
                    } else {
                        info!("[VIDEO] Error reading frame: {e}");
                    }
                }
            }
            info!("[VIDEO][TIME] Reading bytes {:?}", timestart.elapsed());
            frame_counter += 1;
            debug!("[VIDEO] Bufsize: Read {} of expected {}", buffer.len(), bytes );
            // Convert the read bytes into a image and sort it
            let mut frame = RgbImage::from_raw(w, h, buffer.clone()).expect("Could not read data into image format");
            self.sort(&mut frame);
            // Write the sorted image out to the second ffmpeg process
            out_pipe.write(frame.as_raw().as_slice())?;
        }
        Ok(())

    }

    pub fn mask(&self, img: &mut RgbImage) -> bool {
        let mut all_pixels: Vec<&mut Rgb<u8>> = img.pixels_mut().collect();
        if let PixelSelector::Threshold { min, max, criteria } = self.selector {
            self.selector.mask(&mut all_pixels);
            return true;
        }
        return false;
    }
}
