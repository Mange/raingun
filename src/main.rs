use std::path::Path;

extern crate image;

extern crate time;
use time::PreciseTime;

#[macro_use]
extern crate clap;
use clap::Arg;

extern crate raingun_lib as raingun;
use raingun::*;
use raingun::material::{Material, Coloration, Texture, Surface};

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const DEFAULT_WIDTH_STR: &'static str = "800";
const DEFAULT_HEIGHT_STR: &'static str = "600";

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
                 .default_value("test1.png"))
        .get_matches();

    let blue_marble = image::open(&Path::new("./textures/land_ocean_ice_cloud_2048.jpg")).expect("Could not load texture");
    let clay_ground = image::open(&Path::new("./textures/clay-ground-seamless.jpg")).expect("Could not load texture");

    let scene = Scene {
        default_color: Color {
            red: 0.4,
            green: 0.5,
            blue: 1.0,
        },
        max_recursion_depth: 10,
        width: matches
            .value_of("width")
            .unwrap()
            .parse()
            .unwrap_or(DEFAULT_WIDTH),
        height: matches
            .value_of("height")
            .unwrap()
            .parse()
            .unwrap_or(DEFAULT_HEIGHT),
        fov: 90.0,
        bodies: vec![Body::Plane(Plane {
                                     origin: Point3::new(0.0, -2.0, -5.0),
                                     normal: Vector3::new(0.0, -1.0, 0.0),
                                     material: Material {
                                         coloration: Coloration::Texture(Texture {
                                                                             image: clay_ground,
                                                                             x_offset: 0.0,
                                                                             y_offset: 0.0,
                                                                         }),
                                         albedo: 0.15,
                                         surface: Surface::Diffuse,
                                     },
                                 }),
                     Body::Plane(Plane {
                                     origin: Point3::new(0.0, 0.0, -40.0),
                                     normal: Vector3::new(0.0, 0.0, -1.0),
                                     material: Material {
                                         coloration: Coloration::Color(Color {
                                                                           red: 0.4,
                                                                           green: 0.5,
                                                                           blue: 1.0,
                                                                       }),
                                         albedo: 0.9,
                                         surface: Surface::Diffuse,
                                     },
                                 }),
                     Body::Sphere(Sphere {
                                      center: Point3::new(0.0, 0.0, -5.0),
                                      radius: 1.0,
                                      material: Material {
                                          coloration: Coloration::Texture(Texture {
                                                                              image: blue_marble,
                                                                              x_offset: 0.9,
                                                                              y_offset: 0.0,
                                                                          }),
                                          albedo: 0.6,
                                          surface: Surface::Diffuse,
                                      },
                                  }),
                     Body::Sphere(Sphere {
                                      center: Point3::new(-10.0, 3.0, -15.2),
                                      radius: 5.0,
                                      material: Material {
                                          coloration: Coloration::Color(Color {
                                                                            red: 1.0,
                                                                            green: 1.0,
                                                                            blue: 1.0,
                                                                        }),
                                          albedo: 0.5,
                                          surface: Surface::Reflecting { reflectivity: 0.75 },
                                      },
                                  }),
                     Body::Sphere(Sphere {
                                      center: Point3::new(0.0, 3.7, -8.2),
                                      radius: 2.0,
                                      material: Material {
                                          coloration: Coloration::Color(Color {
                                                                            red: 1.0,
                                                                            green: 1.0,
                                                                            blue: 0.8,
                                                                        }),
                                          albedo: 0.5,
                                          surface: Surface::Diffuse,
                                      },
                                  }),
                     Body::Sphere(Sphere {
                                      center: Point3::new(2.0, 1.0, -6.0),
                                      radius: 1.5,
                                      material: Material {
                                          coloration: Coloration::Color(Color {
                                                                            red: 1.0,
                                                                            green: 0.95,
                                                                            blue: 0.95,
                                                                        }),
                                          albedo: 0.18,
                                          surface: Surface::Refractive {
                                              index: 1.33, // Water at 20 ℃
                                              transparency: 0.8,
                                          },
                                      },
                                  })],
        lights: vec![Light::Directional(DirectionalLight {
                                            direction: Vector3::new(0.4, -1.0, -0.9),
                                            color: Color {
                                                red: 1.0,
                                                green: 1.0,
                                                blue: 0.7,
                                            },
                                            intensity: 7.0,
                                        }),
                     Light::Spherical(SphericalLight {
                                          position: Point3::new(-6.0, 3.2, -5.0),
                                          color: Color {
                                              red: 0.9,
                                              green: 0.0,
                                              blue: 0.7,
                                          },
                                          intensity: 4000.0,
                                      }),
                     Light::Spherical(SphericalLight {
                                          position: Point3::new(30.0, 20.0, -30.0),
                                          color: Color {
                                              red: 1.0,
                                              green: 1.0,
                                              blue: 0.7,
                                          },
                                          intensity: 15000.0,
                                      })],
    };

    let start = PreciseTime::now();
    let image = render(&scene);
    let end = PreciseTime::now();

    println!("Took {} to render scene", start.to(end));

    image
        .save(&Path::new(matches.value_of("output").unwrap()))
        .expect("Could not encode image");
}
