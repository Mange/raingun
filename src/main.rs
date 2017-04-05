use std::path::{Path, PathBuf};

extern crate image;

extern crate time;
use time::PreciseTime;

#[macro_use]
extern crate clap;
use clap::Arg;

extern crate raingun_lib as raingun;
use raingun::Scene;

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const DEFAULT_WIDTH_STR: &'static str = "800";
const DEFAULT_HEIGHT_STR: &'static str = "600";

extern crate serde_yaml;
use std::fs::File;

fn main() {
    let matches = app_from_crate!()
        .arg(Arg::with_name("width")
                 .short("w")
                 .long("width")
                 .value_name("PIXELS")
                 .help("Width of output image.")
                 .default_value(DEFAULT_WIDTH_STR))
        .arg(Arg::with_name("height")
                 .short("h")
                 .long("height")
                 .value_name("PIXELS")
                 .help("Height of output image.")
                 .default_value(DEFAULT_HEIGHT_STR))
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
        .get_matches();

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
    let scene: Scene = serde_yaml::from_reader(&file).expect("Could not load YAML");

    let width = matches
        .value_of("width")
        .unwrap()
        .parse()
        .unwrap_or(DEFAULT_WIDTH);
    let height = matches
        .value_of("height")
        .unwrap()
        .parse()
        .unwrap_or(DEFAULT_HEIGHT);

    let start = PreciseTime::now();
    let image = scene.render(width, height);
    let end = PreciseTime::now();

    println!("Took {} to render scene", start.to(end));

    image.save(&output_path).expect("Could not encode image");
}
