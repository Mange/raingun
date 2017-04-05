use std::path::Path;

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
                 .default_value(DEFAULT_WIDTH_STR))
        .arg(Arg::with_name("height")
                 .short("h")
                 .long("height")
                 .value_name("PIXELS")
                 .default_value(DEFAULT_HEIGHT_STR))
        .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .value_name("FILENAME")
                 // TODO: Default to input name, but with ".png" as extension.
                 .default_value("test1.png"))
        // TODO: Add a --force option to overwrite existing files.
        .get_matches();

    // TODO: Add as a parameter instead.
    let file = File::open("examples/test1.yml").expect("Could not open example file");
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

    image
        .save(&Path::new(matches.value_of("output").unwrap()))
        .expect("Could not encode image");
}
