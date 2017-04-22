/*
Preview window works with multiple threads:
    - Render threads (multiple)
        Renders one pixel at a time of the image.

    - Collector thread (single)
        Receives rendered pixels on the channel and puts them into an ImageBuffer.

    - Window thread (single)
        Opens a window and renders the ImageBuffer to it occasionally.

The collected image will be locked using a Mutex and communication from render threads to the
collector thread uses an async channel with "unlimited buffer". That means that even if the preview
window has locked the imagebuffer and the collector cannot read the channels, the render threads
can still send new pixel data to the channel.
*/
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread;

extern crate image;
use image::Rgba;
use time::{PreciseTime, Duration};
use parking_lot::{Mutex, RwLock};

use raingun::{Scene, RenderedPixel};

pub type ImageBuffer = image::ImageBuffer<Rgba<u8>, Vec<u8>>;

pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub max_recursion_depth: Option<u32>,
}

impl Default for RenderOptions {
    fn default() -> RenderOptions {
        RenderOptions {
            width: 800,
            height: 600,
            max_recursion_depth: None,
        }
    }
}


pub fn render_image_without_preview(scene: &Scene,
                                    render_options: &RenderOptions,
                                    input_path: &Path,
                                    output_path: &Path) {

    let render_start = PreciseTime::now();
    let image = scene.render_image(render_options.width, render_options.height);
    let render_end = PreciseTime::now();

    image.save(&output_path).expect("Could not encode image");

    let write_end = PreciseTime::now();

    print_render_message(&input_path,
                         &output_path,
                         render_start.to(render_end),
                         render_end.to(write_end));
}

pub fn render_image_with_preview(scene: &Scene,
                                 render_options: &RenderOptions,
                                 input_path: &Path,
                                 output_path: &Path) {
    // Create shared image buffer
    let shared_image: ImageBuffer = image::ImageBuffer::new(render_options.width,
                                                            render_options.height);
    let shared_image = Arc::new(Mutex::new(shared_image));

    // Create render thread channel
    let (channel_tx, channel_rx) = channel();

    // Create window
    let close_window_condition = Arc::new(RwLock::new(false));
    let window_thread = start_window_thread(shared_image.clone(),
                                            render_options,
                                            close_window_condition.clone());

    // Start collector thread
    let collector_thread = start_collector_thread(channel_rx, shared_image.clone());

    // Start rendering threads
    let render_start = PreciseTime::now();
    scene.streaming_render(render_options.width, render_options.height, channel_tx);
    let render_end = PreciseTime::now();

    // When done, close channel
    //    ...on closed channel, close window and stop collector.
    // (This happens in the other threads)

    // Store image buffer to output file when everything is done
    let write_end = {
        let shared_image = shared_image.lock();

        shared_image
            .save(&output_path)
            .expect("Could not encode image");

        PreciseTime::now()
    };

    print_render_message(&input_path,
                         &output_path,
                         render_start.to(render_end),
                         render_end.to(write_end));

    // Exit
    {
        let mut close = close_window_condition.write();
        *close = true;
    }
    collector_thread.join().unwrap();
    window_thread.join().unwrap();
}

fn start_window_thread(shared_image: Arc<Mutex<ImageBuffer>>,
                       render_options: &RenderOptions,
                       close_condition: Arc<RwLock<bool>>)
                       -> JoinHandle<()> {
    let width = render_options.width;
    let height = render_options.height;

    thread::spawn(move || {
        use piston_window::*;
        let mut window: PistonWindow = WindowSettings::new("Raingun", (width, height))
            .exit_on_esc(true)
            .title(String::from("Raingun preview"))
            .vsync(true)
            .resizable(true)
            .decorated(true)
            .controllers(false) // Don't look for gamepad events
            .build()
            .expect("Could not build PistonWindow");

        window.set_ups(15); // Try to update 15 times per second
        window.set_ups_reset(1); // Skip updating when falling behind

        // Don't try to render too fast. We know nothing exciting happens while we wait for an
        // update.
        window.set_max_fps(30);

        let mut texture = {
            let image = shared_image.lock();
            let texture_settings = TextureSettings::new();
            Texture::from_image(&mut window.factory, &image, &texture_settings).unwrap()
        };

        let width_f = width as f64;
        let height_f = height as f64;
        let half_width = width_f / 2.0;
        let half_height = height_f / 2.0;

        while let Some(event) = window.next() {
            if let Some(_) = event.update_args() {
                if *close_condition.read() {
                    window.set_should_close(true);
                }

                let image = shared_image.lock();
                texture.update(&mut window.encoder, &image).unwrap();
            }

            if let Some(r) = event.render_args() {
                window.draw_2d(&event, |context, graphics| {
                    clear([0.0, 0.0, 0.0, 1.0], graphics);

                    // Center image inside window
                    let (center_x, center_y) = ((r.width / 2) as f64, (r.height / 2) as f64);

                    // Resize image to show as much as possible on the screen
                    let ratio = (r.width as f64 / width_f).min(r.height as f64 / height_f);

                    let transform = context
                        .transform
                        .trans(center_x, center_y)
                        .scale(ratio, ratio)
                        .trans(-half_width, -half_height);

                    image(&texture, transform, graphics);
                });
            }
        }
    })
}

fn start_collector_thread(channel_rx: Receiver<RenderedPixel>,
                          shared_image: Arc<Mutex<ImageBuffer>>)
                          -> JoinHandle<()> {
    thread::spawn(move || {
        // rustfmt has a bug when it formats this while loop. It can be solved by having this
        // comment here.
        // See this issue for details: https://github.com/rust-lang-nursery/rustfmt/issues/1467
        loop {
            let message = channel_rx.recv();
            match message {
                Ok(rendered_pixel) => {
                    let mut image = shared_image.lock();
                    image.put_pixel(rendered_pixel.x,
                                    rendered_pixel.y,
                                    rendered_pixel.color.rgba());
                }
                Err(_) => {
                    // Channel was closed, abort collector loop.
                    break;
                }
            }
        }
    })
}

fn print_render_message(input_path: &Path,
                        output_path: &Path,
                        render_duration: Duration,
                        write_duration: Duration) {
    println!("{input}\t→\t{output}\t({render_duration} render, {write_duration} write)",
             input = input_path.to_string_lossy(),
             output = output_path.to_string_lossy(),
             render_duration = format_duration(render_duration),
             write_duration = format_duration(write_duration));
}

fn format_duration(duration: Duration) -> String {
    const ONE_MINUTE: i64 = 1000 * 60;

    let milliseconds = duration.num_milliseconds();
    match milliseconds {
        0...800 => format!("{}ms", milliseconds),
        800...ONE_MINUTE => format!("{:.2}s", milliseconds as f32 / 1000.0),
        n if n < 0 => unreachable!("Time travel discovered. I'm happy we crashed!"),
        _ => {
            let minutes = milliseconds / ONE_MINUTE;
            let ms_left = milliseconds - minutes * ONE_MINUTE;

            format!("{}m {:.2}s", minutes, ms_left as f32 / 1000.0)
        }
    }
}
