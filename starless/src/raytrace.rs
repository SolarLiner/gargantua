
use color::color::Color;
use image::{DynamicImage, GenericImage, Pixel, Rgba};
use nalgebra::{Vector2, Vector3};
use std::f64;

use crate::texture::Texture;
use crate::utils::color_to_rgba;

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
pub struct Scene {
	pub width: u32,
	pub height: u32,
	pub fov: f64,
	pub sphere: Sphere,
}

pub trait Intersectable {
	fn intersect(&self, ray: &Ray) -> Option<f64>;
	fn surface_normal(&self, hit: &Vector) -> Vector;
	fn texture_coords(&self, hit: &Vector) -> TexCoords;
}

impl Ray {
	pub fn create_prime(scn: &Scene, x: u32, y: u32) -> Self {
		let aspect_ratio = (scn.width as f64) / (scn.height as f64);
		let fov_adjust = (scn.fov.to_radians() / 2.0).tan();
		let sensor_x = (((x as f64 + 0.5) / scn.width as f64) * 2.0 - 1.0) * aspect_ratio;
		let sensor_y = 1.0 - ((y as f64 + 0.5) / scn.height as f64) * 2.0;
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
		let vec = *hit - self.pos;
		return TexCoords::new(
			1.0 + (vec.z.atan2(vec.x) as f64) / f64::consts::PI * 0.5,
			(vec.y / self.radius).acos() as f64 / f64::consts::PI,
		);
	}
}

impl Scene {
	pub fn render(&self) -> DynamicImage {
		let mut img = DynamicImage::new_rgba8(self.width, self.height);
		img.as_mut_rgba8()
			.unwrap()
			.enumerate_pixels_mut()
			.for_each(|(x, y, px)| {
				let ray = Ray::create_prime(self, x, y);
				match self.sphere.intersect(&ray) {
					Some(p) => {
						let hit = ray.origin + ray.direction * p;
						let uv = self.sphere.texture_coords(&hit);
						*px = color_to_rgba(&self.sphere.texture.uv(uv));
					}
					None => *px = Rgba::from_channels(0, 0, 0, 0),
				}
			});

		return img;
	}
}
