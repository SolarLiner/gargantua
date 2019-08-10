use color::Color;
use image::{DynamicImage, Pixel, Rgb, Rgba};
use nalgebra::{Isometry3, Perspective3, Point3, Translation3, UnitQuaternion, Vector2, Vector3};
use std::f64;

use crate::texture::{Texture, TextureFiltering, TextureMode};
use crate::utils::{cartesian_to_spherical, color_to_rgba};

type Vector = Vector3<f64>;
type TexCoords = Vector2<f64>;

#[derive(Clone, Debug)]
pub struct Ray {
	pub origin: Vector,
	pub direction: Vector,
}

#[derive(Clone)]
pub struct Sphere {
	pub pos: Vector,
	pub radius: f64,
	pub texture: Texture,
}

#[derive(Clone)]
pub struct Camera {
	pub width: u32,
	pub height: u32,
	pub isometry: Isometry3<f64>,
	pub perspective: Perspective3<f64>,
}

#[derive(Clone)]
pub struct Scene {
	pub camera: Camera,
	pub sphere: Sphere,
	pub bgtex: Option<Texture>,
}

pub trait Intersectable {
	fn intersect(&self, ray: &Ray) -> Option<f64>;
	fn surface_normal(&self, hit: &Vector) -> Vector;
	fn texture_coords(&self, hit: &Vector) -> TexCoords;
}

pub trait Renderable {
	fn render_px(&self, x: u32, y: u32) -> Color;
	fn get_dimensions(&self) -> (u32, u32);
}

impl Ray {
	pub fn create_prime(width: u32, height: u32, fov: f64, x: u32, y: u32) -> Self {
		let aspect_ratio = (width as f64) / (height as f64);
		let fov_adjust = (fov.to_radians() / 2.0).tan();
		let sensor_x = (((x as f64 + 0.5) / width as f64) * 2.0 - 1.0) * aspect_ratio;
		let sensor_y = 1.0 - ((y as f64 + 0.5) / height as f64) * 2.0;
		return Ray {
			origin: Vector::zeros(),
			direction: Vector::new(sensor_x * fov_adjust, sensor_y * fov_adjust, -1.0).normalize(),
		};
	}
}

impl Intersectable for Sphere {
	fn intersect(&self, ray: &Ray) -> Option<f64> {
		let l = self.pos - ray.origin;
		let adj2 = l.dot(&ray.direction);
		let d2 = l.dot(&l) - (adj2 * adj2);
		let r2 = self.radius * self.radius;
		if d2 > r2 {
			return None;
		}

		let thc = (r2 - d2).sqrt();
		let t0 = adj2 - thc;
		let t1 = adj2 + thc;

		if t0 < 0.0 && t1 < 0.0 {
			return None;
		} else if t0 < 0.0 {
			return Some(t1);
		} else if t1 < 0.0 {
			return Some(t0);
		} else {
			let dist = if t0 < t1 { t0 } else { t1 };
			return Some(dist);
		}
	}
	fn surface_normal(&self, hit: &Vector) -> Vector {
		(*hit - self.pos).normalize()
	}
	fn texture_coords(&self, hit: &Vector) -> TexCoords {
		let dir = *hit - self.pos;
		let r = dir.dot(&dir).sqrt();
		let phi = dir.y.atan2(dir.x);
		let theta = (dir.z / r).acos();
		return TexCoords::new(theta / f64::consts::FRAC_PI_2, phi / f64::consts::PI);
	}
}

impl Camera {
	pub fn new(width: u32, height: u32, fov: f64) -> Self {
		Self {
			width,
			height,
			perspective: Perspective3::new(height as f64 / width as f64, fov, 0.01, 200.0),
			isometry: Isometry3::identity(),
		}
	}

	pub fn transform(&self, vec: &Vector) -> Vector {
		let pt = Point3::from(*vec);
		let transformed = self
			.perspective
			.project_point(&self.isometry.transform_point(&pt));

		return transformed.coords;
	}

	pub fn rotate(&self, vec: &Vector) -> Vector {
		self.perspective
			.project_vector(&self.isometry.transform_vector(vec))
	}

	pub fn transform_ray(&self, ray: &Ray) -> Ray {
		let origin = self.transform(&ray.origin);
		let direction = self.rotate(&ray.direction);

		Ray { origin, direction }
	}

	pub fn create_primary(&self, x: u32, y: u32) -> Ray {
		let origin = Vector::new(
			2.0 * x as f64 / self.width as f64 - 1.0,
			2.0 * y as f64 / self.height as f64 - 1.0,
			0.0,
		);
		let direction = Vector::new(0.0, 0.0, -1.0);

		self.transform_ray(&Ray { origin, direction })
	}

	pub fn set_position(&mut self, pos: Translation3<f64>) {
		self.isometry = Isometry3::from_parts(pos, self.isometry.rotation);
	}

	pub fn set_rotation(&mut self, rot: UnitQuaternion<f64>) {
		self.isometry = Isometry3::from_parts(self.isometry.translation, rot);
	}
}

impl Scene {
	pub fn set_camera(
		&mut self,
		trans: Option<Translation3<f64>>,
		rot: Option<UnitQuaternion<f64>>,
		fov: Option<f64>,
	) {
		self.camera.isometry = match (trans, rot) {
			(Some(t), Some(r)) => Isometry3::from_parts(t, r),
			(None, _) | (_, None) => self.camera.isometry,
		};
		if let Some(f) = fov {
			self.camera.perspective.set_fovy(f);
		}
	}

	pub fn set_size(&mut self, width: u32, height: u32) {
		self.camera.width = width;
		self.camera.height = height;
		self.camera
			.perspective
			.set_aspect(width as f64 / height as f64);
	}

	pub fn get_background(mut self) -> Texture {
		if let Some(tex) = self.bgtex {
			return tex;
		} else {
			self.bgtex = Self::create_bg_texture(50, 50).ok();
			return self.bgtex.expect("Can't create background texture");
		}
	}

	pub fn create_bg_texture(width: u32, height: u32) -> Result<Texture, &'static str> {
		let mut bg_img = DynamicImage::new_rgb8(width, height);

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
}

impl Renderable for Scene {
	fn render_px(&self, x: u32, y: u32) -> Color {
		let this = self.clone();
		let bgtex = this.get_background();
		let ray = self.camera.create_primary(x, y);
		match self.sphere.intersect(&ray) {
			Some(p) => {
				let hit = ray.origin + ray.direction * p;
				let uv = self.sphere.texture_coords(&hit);
				return self.sphere.texture.uv(uv);
			}
			None => {
				let (_, theta, phi) = cartesian_to_spherical(&ray.direction);
				let uv = Vector2::new(theta / f64::consts::PI, phi / f64::consts::FRAC_PI_2);
				return bgtex.uv(uv);
			}
		}
	}

	fn get_dimensions(&self) -> (u32, u32) {
		(self.camera.width, self.camera.height)
	}
}

pub mod render {
	use super::Renderable;

	use crate::utils::{color_to_rgba, DimIterator};
	use image::{DynamicImage, GenericImageView};
	// use rayon::prelude::*;
	use rayon::ThreadPoolBuilder;

	use std::sync::{mpsc, Arc, Mutex};

	type Reporter<'a> = &'a Fn(f64, String);

	pub fn render<'a, R: Renderable + Clone + Send + Sync + 'static>(
		o: R,
		r: Option<Reporter<'a>>,
	) -> Result<DynamicImage, &'static str> {
		let (width, height) = o.get_dimensions();
		let num_threads = num_cpus::get().min(30); // Set an upper bound on the number of threads to not overwhelm the OS
		let chunk_size = 32u32;
		let num_columns = 1 + width / chunk_size;
		let num_rows = 1 + height / chunk_size;

		let pool = ThreadPoolBuilder::new()
			.num_threads(num_threads)
			.build()
			.or(Err("Cannot setup threading"))?;
		let (tx, rx) = mpsc::channel();

		let osrc = Arc::new(o);
		let misses = Arc::new(Mutex::new(0u32));

		for cy in 0..num_rows {
			for cx in 0..num_columns {
				let x = chunk_size * cx;
				let y = chunk_size * cy;
				let x_size = chunk_size.min(width - x);
				let y_size = chunk_size.min(height - y);

				let ttx = tx.clone();
				let this = Arc::clone(&osrc);
				let m = Arc::clone(&misses);

				pool.spawn(move || {
					for (x, y) in DimIterator::create(x_size, y_size, x, y) {
						match ttx.send((x, y, this.render_px(x, y))) {
							Ok(_) => (),
							Err(_) => {
								let mut mref = m.lock().unwrap();
								*mref += 1;
							}
						}
					}
				})
			}
		}

		drop(tx);

		let mut img = DynamicImage::new_rgba8(width, height);
		match img.as_mut_rgba8() {
			Some(buf) => {
				let mut i: usize = 0;
				let tot = width * height;
				for (x, y, col) in rx.into_iter() {
					if let Some(f) = r {
						if i % 40 == 0 {
							let nm = *misses.lock().unwrap();
							if nm > 0 {
								(*f)(
									i as f64 / tot as f64,
									format!("Raytracing ({} missed/overshot pixels)...", nm),
								);
							} else {
								(*f)(i as f64 / tot as f64, format!("Raytracing..."));
							}
						}
					}
					i += 1;
					if buf.in_bounds(x, y) {
						buf.put_pixel(x, y, color_to_rgba(&col));
					} else {
						*misses.lock().unwrap() += 1;
					}
				}
				let num_misses = *misses.lock().unwrap();
				if num_misses > 0 {
					println!("WARNING: Missed {} pixels", num_misses);
				}
				Ok(img)
			}
			None => Err("Couldn't create image"),
		}
	}
}
/*
#[cfg(test)]
mod tests {
	use super::Camera;
	use nalgebra::{Translation3, Vector3};

	#[test]
	fn camera_works() {
		let mut cam = Camera::new(500, 500, 50.0);
		cam.set_position(Translation3::new(0.0, 0.0, 10.0));

		assert_relative_eq!(
			cam.transform(&Vector3::zeros()),
			Vector3::new(0.0, 0.0, -10.0)
		);
	}

	#[test]
	fn camera_creates_primary() {
		let cam = Camera::new(500, 500, 50.0);
		let ray = cam.create_primary(250, 250);

		println!("{:?}", ray);
		assert_relative_eq!(ray.origin, Vector3::new(0.0, 0.0, 0.0));
	}
}
 */
