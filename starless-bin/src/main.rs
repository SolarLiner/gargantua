use image::{DynamicImage, Pixel, Rgb};
use nalgebra::{Translation3, Vector3};
use regex::Regex;

use starless::raytrace::render::render;
use starless::{Camera, GRScene, Scene, Sphere, Texture, TextureFiltering, TextureMode};

use std::{f64, u32};

type Vector = Vector3<f64>;

enum SpaceTime {
	Flat,
	Schwardzchild,
}

fn create_sphere_texture() -> Texture {
	let mut img = DynamicImage::new_rgb8(64, 64);
	for (x, y, p) in img.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
		*p = if (x + y) % 2 == 0 {
			Rgb::from_channels(255, 0, 100, 255)
		} else {
			Rgb::from_channels(100, 0, 255, 255)
		};
	}

	return Texture(img, TextureFiltering::Nearest, TextureMode::Transparent);
}

fn setup_scene_flat(w: u32, h: u32) -> Scene {
	let mut scn = Scene {
		camera: Camera::new(w, h, 10.0),
		sphere: Sphere {
			pos: Vector::zeros(),
			radius: 1.0,
			texture: create_sphere_texture(),
		},
		bgtex: None,
	};
	scn.set_camera(Some(Translation3::new(0.0, 0.0, 20.0)), None, None);

	return scn;
}

fn setup_scene_gr(w: u32, h: u32) -> GRScene {
	GRScene(setup_scene_flat(w, h), 0.16, 300)
}

fn main() {
	let size_re = Regex::new(r"(\d+)x(\d+)").expect("Couldn't create regex");
	let matches = clap::App::new("Starless (now Rusty!)")
		.version("0.1")
		.author("Nathan Graule <solarliner@gmail.com>")
		.about("Render black hole in Flat (boring) or Schwardzchild (awesome) spacetime.")
		.arg(clap::Arg::with_name("OUT").help("Output filename"))
		.arg(
			clap::Arg::with_name("size")
				.short("s")
				.value_name("WIDTHxHEIGHT")
				.help("Sets the output image size")
				.takes_value(true),
		)
		.arg(
			clap::Arg::with_name("quiet")
				.short("q")
				.help("Quiet output (no progress readout)"),
		)
		.subcommand(
			clap::SubCommand::with_name("flat").about("Renders a black hole in flat spacetime"),
		)
		.subcommand(
			clap::SubCommand::with_name("warped").about("Renders scene in Schwardzchild spacetime"),
		)
		.get_matches();

	let st_type = if let Some(subcommand) = matches.subcommand_name() {
		match subcommand {
			"flat" => SpaceTime::Flat,
			_ => SpaceTime::Schwardzchild,
		}
	} else {
		SpaceTime::Schwardzchild
	};
	let (width, height) = matches
		.value_of("size")
		.and_then(|s| size_re.captures(s))
		.map(|c| {
			(
				u32::from_str_radix(&c[1], 10).expect("Couldn't parse width"),
				u32::from_str_radix(&c[2], 10).expect("Couldn't parse height"),
			)
		})
		.unwrap_or((640u32, 360u32));

	run(
		st_type,
		matches.value_of("output").unwrap_or("output.png"),
		width,
		height,
		matches.is_present("quiet"),
	);
}

fn run(st_type: SpaceTime, output: &str, width: u32, height: u32, quiet: bool) {
	if !quiet {
		println!("Rendering a {:?} image to {}", (width, height), output);
	}
	let start = std::time::Instant::now();
	let report =
		|p: f64, msg: String| print!("{}               \r", progressbar(30, Some(start), p, msg));

	match st_type {
		SpaceTime::Flat => {
			let scene = setup_scene_flat(width, height);
			render(scene, if quiet { None } else { Some(&report) })
				.map(|i| i.save(output).expect("Error saving image"))
				.expect("Error rendering image");
		}
		SpaceTime::Schwardzchild => {
			let scene = setup_scene_gr(width, height);
			render(scene, if quiet { None } else { Some(&report) })
				.map(|i| i.save(output).expect("Error saving image"))
				.expect("Error rendering image");
		}
	}

	if quiet {
		println!(
			"Done in {:.2} s.",
			start.elapsed().as_millis() as f64 / 1000.0
		);
	} else {
		report(
			1.0,
			format!(
				"Done in {:.2} s.",
				start.elapsed().as_millis() as f64 / 1000.0
			),
		);
	}
	print!("\n");
}

fn progressbar(width: u8, start: Option<std::time::Instant>, p: f64, msg: String) -> String {
	let filled = (width as f64 * p).round() as u8;
	(0..filled)
		.map(|_| '=')
		.chain(">".chars())
		.chain((filled..width).map(|_| ' '))
		.chain({
			if let Some(s) = start {
				let elapsed = s.elapsed().as_millis() as f64 / 1000.0;
				let eta = if p > 1e-4 {
					(1.0 - p) / p * elapsed
				} else {
					f64::INFINITY
				};
				format!(" [ETA {:.2} s - {:2.1} %] {}", eta, 100.0 * p, msg)
			} else {
				format!(" [{:2.1} %] {}", 100.0 * p, msg)
			}
			.chars()
		})
		.collect()
}
