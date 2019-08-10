#[macro_use]
extern crate criterion;

use criterion::{Criterion, ParameterizedBenchmark};

use image::{DynamicImage, Pixel, Rgb};
use nalgebra::{Translation3};
use starless::{render, Camera, GRScene, Scene, Sphere, Texture, TextureFiltering, TextureMode};
use starless::raytrace::{Point};

use rand::Rng;

fn create_bg_texture() -> Texture {
	let mut rng = rand::thread_rng();
	let mut tex = DynamicImage::new_rgb8(200, 200);
	for (_x, _y, p) in tex.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
		*p = if rng.gen::<f32>() < 0.1 {
			Rgb::from_channels(255, 255, 255, 255)
		} else {
			Rgb::from_channels(0, 0, 0, 0)
		};
	}

	return Texture(tex, TextureFiltering::Bilinear, TextureMode::Repeat);
}

fn create_sphere_texture() -> Texture {
	let mut tex = DynamicImage::new_rgb8(50, 50);
	for (x, y, p) in tex.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
		*p = if (x + y) % 2 == 0 {
			Rgb::from_channels(0, 0, 0, 0)
		} else {
			Rgb::from_channels(255, 255, 255, 255)
		};
	}

	return Texture(tex, TextureFiltering::Nearest, TextureMode::Repeat);
}

fn setup_scene_flat(w: u32, h: u32) -> Scene {
	let mut scn = Scene {
		bgtex: Some(create_bg_texture()),
		camera: Camera::new(w, h, 45.0),
		sphere: Sphere {
			pos: Point::new(0.0, 0.0, 0.0),
			radius: 1.0,
			texture: create_sphere_texture(),
		},
	};

	scn.set_camera(Some(Translation3::new(0.0, 0.0, 20.0)), None, None);

	return scn;
}

fn setup_scene_gr(max_iter: u32) -> GRScene {
	GRScene(setup_scene_flat(100, 100), 0.16, max_iter)
}

fn crit_bench_flat(c: &mut Criterion) {
	c.bench(
		"scene flat",
		ParameterizedBenchmark::new(
			"scene flat",
			|b, &(w, h)| {
				let scn = setup_scene_flat(w, h);
				b.iter(|| render(scn.clone(), None));
			},
			vec![
				(10, 10),
				(32, 32),
				(50, 50),
				(64, 32),
				(64, 64),
				(100, 100),
				(128, 96),
				(320, 144),
			],
		)
		.measurement_time(std::time::Duration::from_secs(20)),
	);
}

fn crit_bench_gr(c: &mut Criterion) {
	c.bench(
		"scene gr",
		ParameterizedBenchmark::new(
			"scene gr",
			|b, &iter| {
				let scn = setup_scene_gr(iter);
				b.iter(|| render(scn.clone(), None));
			},
			vec![10, 30, 50, 100, 300, 500],
		)
		.measurement_time(std::time::Duration::from_secs(20)),
	);
}

fn bench(c: &mut Criterion) {
	crit_bench_flat(c);
	crit_bench_gr(c);
}

criterion_group!(benches, bench);
criterion_main!(benches);
