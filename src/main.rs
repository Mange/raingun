use std::path::{Path, PathBuf};

extern crate image;

extern crate time;
use time::{PreciseTime, Duration};

#[macro_use]
extern crate clap;
use clap::Arg;

extern crate raingun_lib as raingun;
use raingun::Scene;

extern crate serde_yaml;
use std::fs::File;

fn construct_app<'a, 'b>() -> clap::App<'a, 'b> {
    app_from_crate!()
        .arg(Arg::with_name("width")
                 .short("w")
                 .long("width")
                 .value_name("PIXELS")
                 .help("Width of output image."))
        .arg(Arg::with_name("height")
                 .short("h")
                 .long("height")
                 .value_name("PIXELS")
                 .help("Height of output image."))
        .arg(Arg::with_name("4k")
             .long("4k")
             .help("Renders in 4K resolution. Explicit width/height overrides.")
             .overrides_with("hd")
        )
        .arg(Arg::with_name("hd")
             .long("hd")
             .help("Renders in 1080 (HD) resolution. Explicit width/height overrides.")
             .overrides_with("4k")
        )
        .arg(Arg::with_name("preview")
             .long("preview")
             .help("Renders in 800x600 and lower quality settings.")
             .overrides_with("4k")
             .overrides_with("hd")
             .overrides_with("width")
             .overrides_with("height")
        )
        .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .value_name("FILENAME")
                 .help("Specify filename of the rendered image.")
                 .long_help(
                     "Where to save the rendered image. Defaults to input filename with \".png\" \
                     extension."
                 ))
        .arg(Arg::with_name("input")
                 .value_name("FILE")
                 .required(true)
                 .help("The scene definition file, in YAML format."))
}


struct RenderOptions {
    width: u32,
    height: u32,
    max_recursion_depth: Option<u32>,
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

impl<'a, 'b> From<&'b clap::ArgMatches<'a>> for RenderOptions {
    fn from(matches: &clap::ArgMatches<'a>) -> RenderOptions {
        let mut options = RenderOptions::default();

        if matches.is_present("preview") {
            options.max_recursion_depth = Some(4);
        } else if matches.is_present("hd") {
            options.width = 1920;
            options.height = 1080;
        } else if matches.is_present("4k") {
            options.width = 3840;
            options.height = 2160;
        }

        if let Some(width) = matches.value_of("width") {
            let width = width.parse().expect("Could not parse width");
            options.width = width;
        }

        if let Some(height) = matches.value_of("height") {
            let height = height.parse().expect("Could not parse height");
            options.height = height;
        }

        options
    }
}

fn main() {
    let matches = construct_app().get_matches();
    let render_options = RenderOptions::from(&matches);

    let input_path = Path::new(matches.value_of("input").unwrap());
    let output_path = match matches.value_of("output") {
        Some(path) => PathBuf::from(path),
        None => {
            let mut path = input_path.to_path_buf();
            if !path.set_extension("png") {
                println!("Could not guess output filename from {}",
                         input_path.to_string_lossy());
                ::std::process::exit(2);
            }
            path
        }
    };

    let file = File::open(input_path).expect("Could not open input file");
    let scene = {
        let mut scene: Scene = serde_yaml::from_reader(&file).expect("Could not load YAML");
        if let Some(limit) = render_options.max_recursion_depth {
            if limit < scene.max_recursion_depth {
                scene.max_recursion_depth = limit;
            }
        }
        scene
    };

    let render_start = PreciseTime::now();
    let image = scene.render(render_options.width, render_options.height);
    let render_end = PreciseTime::now();

    image.save(&output_path).expect("Could not encode image");
    let write_end = PreciseTime::now();

    println!("{input}\t→\t{output}\t({render_duration} render, {write_duration} write)",
             input = input_path.to_string_lossy(),
             output = output_path.to_string_lossy(),
             render_duration = format_duration(render_start.to(render_end)),
             write_duration = format_duration(render_end.to(write_end)));
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

#[cfg(test)]
mod test {
    use super::*;

    fn parse_arguments<'a>(args: &[&'static str]) -> clap::ArgMatches<'a> {
        let app = construct_app();
        app.get_matches_from(args)
    }

    #[test]
    fn it_parses_resolution_arguments() {
        let matches = parse_arguments(&["x", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 800);
        assert_eq!(render_options.height, 600);

        let matches = parse_arguments(&["x", "--width", "640", "--height", "480", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 640);
        assert_eq!(render_options.height, 480);

        let matches = parse_arguments(&["x", "--hd", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 1920);
        assert_eq!(render_options.height, 1080);

        let matches = parse_arguments(&["x", "--hd", "--4k", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 3840);
        assert_eq!(render_options.height, 2160);

        let matches = parse_arguments(&["x", "--hd", "--width", "2000", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 2000);
        assert_eq!(render_options.height, 1080);
    }

    #[test]
    fn it_parses_preview_argument() {
        let matches = parse_arguments(&["x", "--hd", "--width", "2000", "--preview", "file"]);
        let render_options = RenderOptions::from(&matches);
        assert_eq!(render_options.width, 800);
        assert_eq!(render_options.height, 600);
        assert_eq!(render_options.max_recursion_depth, Some(4));
    }
}
