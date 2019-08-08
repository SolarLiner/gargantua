use crate::physics::Particle;
use crate::raytrace::{Intersectable, Ray, Scene, Sphere};
use crate::texture::{Texture, TextureFiltering, TextureMode};
use crate::utils::color_to_rgba;
use crate::utils::DimIterator;

use color::Color;
use image::{DynamicImage, Pixel, Rgb, Rgba};
use nalgebra::{Vector2, Vector3};
use rayon::prelude::*;

use std::collections::HashSet;

use std::f64;
use std::sync::{Arc, Mutex};
type Vector = Vector3<f64>;

pub struct GRParticle {
	particle: Particle,
	dt: f64,
}

#[derive(Clone)]
pub struct GRScene(Scene);

impl GRParticle {
	pub fn new(pos: Vector, dt: f64) -> Self {
		GRParticle {
			particle: Particle::new(pos),
			dt,
		}
	}

	pub fn from_particle(part: &Particle, dt: f64) -> Self {
		GRParticle {
			particle: part.clone(),
			dt,
		}
	}

	pub fn intersect(&mut self, sphere: &Sphere, max_iter: u32) -> Option<Vector> {
		for _ in 0..max_iter {
			let from_sphere = self.particle.pos() - sphere.pos;
			self.particle.add_force(gr_potential(from_sphere, 1.0));
			self.particle.update(self.dt);
			let to_sphere = sphere.pos - self.particle.pos();
			if to_sphere.dot(&to_sphere) < sphere.radius * sphere.radius {
				return Some(self.particle.pos());
			}
		}

		return None;
	}
}

impl GRScene {
	pub fn new(scene: Scene) -> Self {
		Self(scene)
	}

	pub fn render(
		&self,
		dt: f64,
		max_iter: u32,
		reporter: Option<&'static Fn(f64, String)>,
	) -> Result<DynamicImage, &'static str> {
		let mut img = DynamicImage::new_rgba8(self.0.width, self.0.height);

		if let Some(f) = reporter {
			(*f)(0.0, String::from("Creating background image..."));
		}

		let bgtex = Self::create_bg_texture(50, 50)?;

		match img.as_mut_rgba8() {
			Some(buf) => {
				for (i, (x, y, p)) in buf.enumerate_pixels_mut().enumerate() {
					if let Some(f) = reporter {
						if i % 10 == 0 {
							let pos = self.0.width * y + x;
							let total = self.0.height * self.0.width;
							let percent = pos as f64 / total as f64;
							(*f)(percent, String::from("Raytracing..."));
						}
						*p = color_to_rgba(&self.render_px(x, y, dt, max_iter, &bgtex));
					}
				}
				if let Some(f) = reporter {
					(*f)(1.0, String::from("Done."));
				}
				Ok(img)
			}
			None => Err("Couldn't create image"),
		}
	}

	pub fn render_threading(
		&self,
		dt: f64,
		max_iter: u32,
		reporter: Option<&'static Fn(f64, String)>,
	) -> Result<DynamicImage, &'static str> {
		let num_threads = 9; // Logical cores + 1
		let num_columns = (num_threads as f32).sqrt().ceil() as u32;
		let num_rows = (num_threads as f64 / num_columns as f64).sqrt().ceil() as u32;
		let chunk_width = self.0.width / num_columns;
		let chunk_height = self.0.height / num_rows;
		let bgtex = Self::create_bg_texture(50, 50)?;

		let pool = rayon::ThreadPoolBuilder::new()
			.num_threads(num_threads)
			.build()
			.or(Err("Cannot setup threading"))?;
		let (tx, rx) = std::sync::mpsc::channel();
		println!("DEBUG: chunk size {:?}", (chunk_width, chunk_height));
		for ww in 0..num_columns {
			for hh in 0..num_rows {
				let SW = chunk_width * ww;
				let SE = chunk_height * hh;
				let NW = (chunk_width * (ww + 1)).min(self.0.width);
				let NE = (chunk_height * (hh + 1)).min(self.0.height);
				println!("DEBUG: {:?} for {:?} to {:?}", (ww, hh), (SW, SE), (NW, NE));
				let bgtex_arc = Arc::from(bgtex.clone());
				let self_arc = Arc::from(self.clone());
				let ttx = tx.clone();
				pool.spawn(move || {
					for (x, y) in DimIterator::create(NW, NE, SW, SE) {
						let this = self_arc.clone();
						let bgtex = bgtex_arc.clone();
						ttx.send((x, y, this.render_px(x, y, dt, max_iter, &bgtex)))
							.ok();
					}
				});
			}
		}
		drop(tx);

		let mut img = DynamicImage::new_rgba8(self.0.width, self.0.height);
		match img.as_mut_rgba8() {
			Some(buf) => {
				let mut i: usize = 0;
				let tot: u32 = self.0.width * self.0.height;
				let mut set = HashSet::new();
				for (x, y, col) in rx.into_iter() {
					if let Some(f) = reporter {
						let overdrawing = if set.insert((x, y)) {
							""
						} else {
							"(OVERDRAWING)"
						};
						(*f)(i as f64 / tot as f64, format!("Raytracing {}", overdrawing));
					}
					i += 1;
					buf.put_pixel(x, y, color_to_rgba(&col));
				}
				Ok(img)
			}
			None => Err("Couldn't create image"),
		}
	}

	fn create_bg_texture(w: u32, h: u32) -> Result<Texture, &'static str> {
		let mut bg_img = DynamicImage::new_rgb8(w, h);

		match bg_img.as_mut_rgb8() {
			Some(buf) => {
				for (x, y, p) in buf.enumerate_pixels_mut() {
					if (x + y) % 2 == 0 {
						*p = Rgb::from_channels(255, 255, 0, 255);
					} else {
						*p = Rgb::from_channels(0, 255, 255, 255);
					}
				}
			}
			None => return Err("Couldn't create background texture"),
		}

		let bgtex = Texture(bg_img, TextureFiltering::Nearest, TextureMode::Repeat);
		return Ok(bgtex);
	}

	fn render_px(&self, x: u32, y: u32, dt: f64, max_iter: u32, bg: &Texture) -> Color {
		let mut part =
			GRParticle::from_particle(&Particle::from_ray(&Ray::create_prime(&self.0, x, y)), dt);
		return part
			.intersect(&self.0.sphere, max_iter)
			.map(|v: Vector| {
				let uv = self.0.sphere.texture_coords(&v);
				return self.0.sphere.texture.uv(uv);
			})
			.or_else(|| {
				let dir = part.particle.vel();
				let r = dir.dot(&dir).sqrt();
				let phi = dir.y.atan2(dir.x);
				let theta = (dir.z / r).acos();

				let uv = Vector2::new(theta / f64::consts::PI * 2.0, phi / f64::consts::PI);

				return Some(bg.uv(uv));
			})
			.unwrap_or(Color::from_u32(0));
	}
}

fn gr_potential(pos: Vector, h2: f64) -> Vector {
	let pos_fifth = pos.dot(&pos).powf(2.5);

	return -1.5 * h2 * pos / pos_fifth;
}

#[test]
fn can_render_schwardzchild() {
	let mut img = DynamicImage::new_rgb8(64, 64);
	for (x, y, p) in img.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
		*p = if (x + y) % 2 == 0 {
			Rgb::from_channels(0, 0, 0, 0)
		} else {
			Rgb::from_channels(255, 255, 255, 0)
		};
	}
	let scene = GRScene::new(Scene {
		width: 10,
		height: 10,
		fov: 45.0,
		sphere: Sphere {
			pos: Vector::new(0.0, 0.0, -4.0),
			radius: 1.0,
			texture: Texture(img, TextureFiltering::Nearest, TextureMode::Clamp),
		},
	});
	scene
		.render(
			1.0,
			5,
			Some(&|p, msg| print!("[{}%] {}           \r", (1000.0 * p).round() / 10.0, msg)),
		)
		.map(|i: DynamicImage| i.save("scene_gr.png"))
		.expect("saving file")
		.ok();
}
