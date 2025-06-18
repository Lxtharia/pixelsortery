use crate::Pixelsorter;
use ffmpeg_the_third::{format, Rescale};
use image::RgbImage;
use log::{debug, error, info};
use std::{fs::File, io::{self, ErrorKind, Read, Write}, process::{Command, Output, Stdio}, time::Instant};
use ffmpeg_the_third::{self as ffmpeg, codec::{self, Parameters}, frame, media, software::scaling};

struct Transcoder {
    ictx: Option<ffmpeg::format::context::Input>,
    octx: ffmpeg::format::context::Output,
    ist_time_base: ffmpeg::Rational,
    ost_time_base: ffmpeg::Rational,
    encoder: ffmpeg::encoder::Video,
    decoder: ffmpeg::decoder::Video,
    scaler_to_rgb: ffmpeg::software::scaling::context::Context,
    scaler_from_rgb: ffmpeg::software::scaling::context::Context,
    main_stream_index: usize,
    current_frame: u32,
    timer: Instant,
    // enc_opts: ffmpeg::Dictionary,
    // loglevel: log::Level,
}

impl Transcoder {
    fn new(input_path: &str, output_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut enc_opts = ffmpeg::Dictionary::new();
        enc_opts.set("preset", "medium");
        // Copied from https://github.com/shssoichiro/ffmpeg-the-third/blob/master/examples/dump-frames.rs
        ffmpeg::init()?;

        let mut ictx = ffmpeg::format::input(input_path)?;
        let mut octx = ffmpeg::format::output(output_path)?;
        let global_header = octx.format().flags().contains(format::Flags::GLOBAL_HEADER);


        // Print information about the input file
        if log::max_level() >= log::Level::Info { ffmpeg::format::context::input::dump(&ictx, 0, Some(&input_path)); }
        // -------- Open input stream
        let input_stream = ictx.streams()
            .best(ffmpeg::media::Type::Video)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input_stream.index(); // The stream index we want to manipulate

        // Guess the codec from output format and add a stream for that
        let mut codec = ffmpeg::encoder::find(octx.format().codec(output_path, media::Type::Video)).unwrap();

        // Create a corresponding output stream for each input stream
        for (idx, stream) in ictx.streams().enumerate() {
            let mut ost = if idx == video_stream_index {
                octx.add_stream(codec).unwrap()
            } else {
                // Set up for stream copy for non-video stream.
                octx.add_stream(ffmpeg::encoder::find(codec::Id::None)).unwrap()
            };
            ost.set_parameters(stream.parameters());
            // We need to set codec_tag to 0 lest we run into incompatible codec tag
            // issues when muxing into a different container format. Unfortunately
            // there's no high level API to do this (yet).
            unsafe {
                (*ost.parameters_mut().as_mut_ptr()).codec_tag = 0;
            }
        }

        // Set context parallelism
        let mut dec_context = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
        let mut enc_context = ffmpeg::codec::context::Context::new_with_codec(codec);
        // Boost performance, hell yeah!
        if let Ok(parallelism) = std::thread::available_parallelism() {
            dec_context.set_threading(ffmpeg::threading::Config {
                kind: ffmpeg::threading::Type::Frame,
                count: parallelism.get(),
            });
            enc_context.set_threading(ffmpeg::threading::Config {
                kind: ffmpeg::threading::Type::Frame,
                count: parallelism.get(),
            });
        }
        // A decoder. `send_packet()` to it and `receive_frame()` from it
        let mut decoder = dec_context.decoder().video()?;

        // -------- Create encoder and output_stream
        let ist_time_base = input_stream.time_base();
        let fps = input_stream.avg_frame_rate();
        // Add new Encoder
        let mut prep_encoder = enc_context
            .encoder()
            .video()?;
        prep_encoder.set_width(decoder.width());
        prep_encoder.set_height(decoder.height());
        prep_encoder.set_aspect_ratio(decoder.aspect_ratio());
        prep_encoder.set_format(decoder.format());
        prep_encoder.set_frame_rate(Some(fps));
        prep_encoder.set_time_base(ist_time_base);
        prep_encoder.set_quality(100);
        if global_header {
            prep_encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }
        // "Open" encoder, whatever that means
        let mut encoder = prep_encoder.open_with(enc_opts).expect("Error opening encoder with supplied settings");

        // Set parameters of output stream
        let mut output_stream = octx.stream_mut(video_stream_index).unwrap();
        output_stream.set_parameters(Parameters::from(&encoder));
        output_stream.set_rate(fps); // Just for metadata

        // Do some internal stuff. This sets time base for the streams
        octx.set_metadata(ictx.metadata().to_owned());
        if log::max_level() >= log::Level::Info { ffmpeg::format::context::output::dump(&octx, 0, Some(&output_path)); }
        octx.write_header().unwrap();

        let ost_time_base = octx.stream(video_stream_index).unwrap().time_base();
        info!("[VIDEO] Video read successfully, modifing stream #{video_stream_index} | {:?}", input_stream);
        //
        // Encoder/Decoder and Stream setup done

        let (width, height, format) = (decoder.width(), decoder.height(), decoder.format());
        // No idea. Somehow important to get good image data
        let mut scaler_to_rgb = scaling::context::Context::get(
            format, width, height,
            ffmpeg::format::Pixel::RGB24, width, height,
            scaling::Flags::BILINEAR,
        )?;
        let mut scaler_from_rgb = scaling::context::Context::get(
            ffmpeg::format::Pixel::RGB24, width, height,
            encoder.format(), encoder.width(), encoder.height(),
            scaling::Flags::BILINEAR,
        )?;

        Ok(Transcoder {
            ictx: Some(ictx),
            octx,
            ist_time_base,
            ost_time_base,
            encoder,
            decoder,
            scaler_to_rgb,
            scaler_from_rgb,
            main_stream_index: video_stream_index,
            timer: Instant::now(),
            current_frame: 0,
            // loglevel: log::Level::Info,
        })
    }

    fn transcode<F>(&mut self, img_filter: F) -> Result<(), ffmpeg::Error>
        where F: Fn(&mut RgbImage)
    {
        // Try to find number of frames, for progress printing
        let mut ictx = self.ictx.take().unwrap(); // Hacky. Take ictx to prevent conflicts of borrowing self twice
        let input_stream = ictx.stream(self.main_stream_index).unwrap();
        let nb_frames = if input_stream.frames() != 0 {
            input_stream.frames()
        } else {
            // Calculate frames if frames() could not be read. Duration might also be 0 though
            let duration = input_stream.duration() as f32;
            ((input_stream.avg_frame_rate().numerator() as f32
                / input_stream.avg_frame_rate().denominator() as f32)
                * duration) as i64
        };
        println!("\n");
        // Go through each data packet and do stuff
        for (stream, mut packet) in ictx.packets().filter_map(Result::ok) {
            if stream.index() == self.main_stream_index {
                self.decoder.send_packet(&packet).unwrap();
                self.receive_and_process_decoded_frames(&img_filter)?;
                self.current_frame += 1;
                Transcoder::print_progress(self.current_frame, nb_frames, self.timer);
            } else {
                // Copy any other stream packets
                packet.rescale_ts(
                    stream.time_base(),
                    self.octx.stream(stream.index()).unwrap().time_base()
                );
                packet.set_position(-1);
                packet.set_stream(stream.index());
                packet.write_interleaved(&mut self.octx).unwrap();
            }
        }


        // Flush encoders and decoders.
        self.decoder.send_eof().unwrap();
        self.receive_and_process_decoded_frames(&img_filter)?;
        self.encoder.send_eof().unwrap();
        self.receive_and_process_encoded_frames();
        self.octx.write_trailer().unwrap();
        println!("");
        Ok(())
    }

    fn receive_and_process_encoded_frames(&mut self) -> Result<(), ffmpeg::Error> {
        // Receive encoded frame
        let mut encoded_packet = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut encoded_packet).is_ok() {
            encoded_packet.set_stream(self.main_stream_index);
            encoded_packet.rescale_ts(self.ist_time_base, self.ost_time_base);
            encoded_packet.write_interleaved(&mut self.octx).unwrap();
        }
        Ok(())
    }

        // Will be called after every packet sent
    fn receive_and_process_decoded_frames<F>(&mut self, img_filter: &F) -> Result<(), ffmpeg::Error>
        where F: Fn(&mut RgbImage)
    {
        let mut decoded_frame = frame::Video::empty();
        while self.decoder.receive_frame(&mut decoded_frame).is_ok() {
            let timestamp = decoded_frame.timestamp();

            info!("Frame: {:?}, {:?}, {:?}", decoded_frame.timestamp(), timestamp, decoded_frame.pts());

            let mut processed_frame = self.manipulate_frame(img_filter, &mut decoded_frame);
            // let mut processed_frame = decoded_frame.clone();

            processed_frame.set_pts(timestamp.or(Some(self.current_frame.into())));
            processed_frame.set_format(decoded_frame.format());
            processed_frame.set_kind(decoded_frame.kind());
            info!("Sorted {:?}, {:?}, {:?}", processed_frame.timestamp(), processed_frame.format(), processed_frame.pts());
            // processed_frame.set_kind(ffmpeg::picture::Type::None);

            // And back into the encoder it goes
            self.encoder.send_frame(&processed_frame).unwrap();
            self.receive_and_process_encoded_frames()?;
        }

        Ok(())
    }

    fn manipulate_frame<F> (&mut self, img_filter: F, in_frame: &frame::Video) -> frame::Video
        where F: Fn(&mut RgbImage)
    {
        // Create rgb frame and rgb image
        let mut rgb_frame = frame::Video::empty();
        self.scaler_to_rgb.run(in_frame, &mut rgb_frame);

        let mut img = frame_to_img(&rgb_frame);
        // Processing frame, yippie!
        img_filter(&mut img);
        // Write image back into frame
        img_to_frame(&img, &mut rgb_frame);

        // Convert rgb back to yuv (or whatever format it was before)
        let mut yuv_frame = frame::Video::empty();
        self.scaler_from_rgb.run(&rgb_frame, &mut yuv_frame);
        yuv_frame
    }

    fn print_progress(current_frame: u32, nb_frames: i64, timer: Instant) {
        // Wonderful progress update code
        let progress = current_frame as f32/nb_frames as f32;
        print!("\r [VIDEO] Processing Frame [{: >5} / {nb_frames}] ({:>3}%) | {:?} elapsed\t| {:?}s left\t",
            current_frame,
            (100.0 * progress) as u32,
            timer.elapsed(),
            if progress == 0.0 {0} else { (timer.elapsed().as_secs() as f32 * ((1.0/progress) - 1.0) ).round() as u32 },
        );
        io::stdout().flush();
    }

}

impl Pixelsorter {

    /// Reads a video stream from a file, sorts every frame and then writes it to another file
    /// Hacky, but hopefully better/faster than my shitty bash script
    pub fn sort_video(&self, input: &str, output: &str) {
        let timer = Instant::now();
        let res = if output == "-" {
            self.stream_sorted_video(input)
        } else {
            self.transcode_sorted_video(input, output)
        };
        match res {
            Ok(_) => {
                println!("\n=== Success! Finished in {:?} !===", timer.elapsed());
            },
            Err(e) => error!("Error sorting video: {e}")
        };
    }

    fn transcode_sorted_video(&self, input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut transcoder = Transcoder::new(input_path, output_path)?;
        Ok(
            transcoder.transcode(|img| self.sort(img))?
        )
    }

    /// Read a video from an input file, sort it and play the result with ffplay
    /// Does not support audio
    fn stream_sorted_video(&self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut ff_out = Command::new("ffplay")
                .arg("-loglevel").arg("error")
                .arg("-pixel_format").arg("rgb24")
                .arg("-f").arg("rawvideo")
                .arg("-video_size").arg(format!("{w}x{h}"))
                .arg("-i").arg("-")
                .stdin(Stdio::piped())
                .spawn().expect("Failed to run ffplay");

        // Read rawvideo from in_pipe, sort the frames, and write it to out_pipe
        let mut in_pipe = ff_in.stdout.take().expect("");
        let mut out_pipe = ff_out.stdin.take().expect("");
        let mut buffer = vec![0u8; bytes as usize];
        let mut frame_counter = 1;
        loop {
            // Read exactly that amount of bytes that make one frame
            print!("\r[VIDEO] Reading Frame [{frame_counter:_>5} / {packet_count}]"); io::stdout().flush();
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
}

/// Opens and decodes a input file, seeks to the frame with the provided timestamp and converts the frame to an image.
/// The time_stamp has to be in the format of the input streams time base.
pub fn extract_video_frame(input_path: &str, frame_time_stamp: f32) -> Result<RgbImage, Box<dyn std::error::Error>> {
    ffmpeg::init().unwrap();
    let mut ictx = ffmpeg::format::input(input_path).unwrap();
    let input_stream = ictx.streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound).unwrap();
    let video_stream_index = input_stream.index(); // The stream index we want to manipulate
    let mut dec_context = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()).unwrap();
    let mut decoder = dec_context.decoder().video().unwrap();
    let (width, height, format) = (decoder.width(), decoder.height(), decoder.format());
    // No idea. Somehow important to get good image data
    let mut scaler_to_rgb = scaling::context::Context::get(
        format, width, height,
        ffmpeg::format::Pixel::RGB24, width, height,
        scaling::Flags::BILINEAR,
    )?;
    for s in ictx.streams() {
        if s.index() != video_stream_index {
            s.discard();
        }
    }
    let position = (frame_time_stamp / (input_stream.time_base().1 as f32 / input_stream.time_base().0 as f32)) as i64;
    info!("Seeking to {}s with timebase {} (= {})", frame_time_stamp, input_stream.time_base(), position);

    ictx.seek(position, position..);
    // TODO: .seek() is broken. Replace it with a manual heuristic to choose the correct frame for a given timestamp
    // TODO: If looping through all packets is slow, load the packets into a hashmap or something
    let mut img = Err(ffmpeg::Error::Unknown);
    'outer: for (stream, mut packet) in ictx.packets().filter_map(Result::ok) {
        if stream.index() == video_stream_index {
            decoder.send_packet(&packet).unwrap();
            let mut decoded_frame = frame::Video::empty();
            while decoder.receive_frame(&mut decoded_frame).is_ok() {
                info!("Extracting frame with timestamp {:?} or {:?}", decoded_frame.pts(), decoded_frame.timestamp());
                let mut rgb_frame = frame::Video::empty();
                scaler_to_rgb.run(&decoded_frame, &mut rgb_frame);
                img = Ok(frame_to_img(&rgb_frame));
                break 'outer;
            }
        }
    }
    Ok(img?)
}

/// Creates a RgbImage from a frame
fn frame_to_img(frame: &frame::Video) -> RgbImage {
    info!("\tTrying to decode frame: {:?}, {}", frame.width(), frame.planes());
    let (w, h, linelength) = (frame.width() as usize, frame.height() as usize, frame.stride(0) as usize);
    let mut data = vec![0; w*h*3];
    let frame_data = frame.data(0);
    for y in 0..h {
        let img_start_of_line = (y * 3 * w);
        let frame_start_of_line = (y * linelength);
        let nb_bytes = (3 * w);
        let dst = &mut data  [img_start_of_line   .. img_start_of_line + nb_bytes];
        let src = &frame_data[frame_start_of_line .. frame_start_of_line + nb_bytes];
        dst.copy_from_slice(src);
    }
    let mut img: RgbImage = RgbImage::from_raw(w as u32, h as u32, data).unwrap();
    img
}

/// Copies pixel data from the image back to the frame.
fn img_to_frame(img: &RgbImage, frame: &mut frame::Video) {
    // This copies the data line by line to take the frames stride into account.
    let (w, h, linelength) = (frame.width() as usize, frame.height() as usize, frame.stride(0) as usize);
    let frame_data = frame.data_mut(0);
    for y in 0..h as usize {
        let img_start_of_line = (y * 3 * w);
        let frame_start_of_line = (y * linelength);
        let nb_bytes = (3 * w);
        let src = &img.as_raw()[ img_start_of_line .. img_start_of_line + nb_bytes];
        let dst = &mut frame_data[frame_start_of_line .. frame_start_of_line + nb_bytes];
        dst.copy_from_slice(src);
    }
}

// For debugging
fn save_frame(data: &[u8],w: u32, h: u32, path: &str) -> std::result::Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    file.write_all(format!("P6\n{} {}\n255\n", w, h).as_bytes())?;
    file.write_all(data)?;
    Ok(())
}
