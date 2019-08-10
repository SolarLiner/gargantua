use crate::physics::Particle;
use crate::raytrace::{Intersectable, Renderable, Scene, Sphere};
use crate::utils::cartesian_to_spherical;

use color::Color;
use image::{DynamicImage, Pixel};
use nalgebra::{Translation3, UnitQuaternion, Vector2, Vector3};
use rayon::prelude::*;

use std::f64;
type Vector = Vector3<f64>;

pub struct GRParticle {
	particle: Particle,
	dt: f64,
}

#[derive(Clone)]
pub struct GRScene(pub Scene, pub f64, pub u32);

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
		let h2vec = self.particle.pos().cross(&self.particle.vel());
		let h2 = h2vec.dot(&h2vec);
		for _ in 0..max_iter {
			let from_sphere = self.particle.pos() - sphere.pos;
			self.particle.add_force(gr_potential(from_sphere, h2));
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
	pub fn get_scene(self) -> Scene {
		self.0
	}

	pub fn set_camera(
		&mut self,
		trans: Option<Translation3<f64>>,
		rot: Option<UnitQuaternion<f64>>,
		fov: Option<f64>,
	) {
		self.0.set_camera(trans, rot, fov);
	}

	pub fn set_size(&mut self, width: u32, height: u32) {
		self.0.set_size(width, height);
	}

}

impl Renderable for GRScene {
	fn render_px(&self, x: u32, y: u32) -> Color {
		let scene = self.0.clone();
		let bg = scene.get_background();
		let mut part = GRParticle::from_particle(
			&Particle::from_ray(&self.0.camera.create_primary(x, y)),
			self.1,
		);
		return part
			.intersect(&self.0.sphere, self.2)
			.map(|v: Vector| {
				let uv = self.0.sphere.texture_coords(&v);
				return self.0.sphere.texture.uv(uv);
			})
			.or_else(|| {
				let (_, theta, phi) = cartesian_to_spherical(&part.particle.vel());

				let uv = Vector2::new(theta / f64::consts::PI, phi / f64::consts::FRAC_PI_2);

				return Some(bg.uv(uv));
			})
			.unwrap_or(Color::from_u32(0));
	}

	fn get_dimensions(&self) -> (u32, u32) {
		return self.0.get_dimensions();
	}
}

fn gr_potential(pos: Vector, h2: f64) -> Vector {
	let pos_fifth = pos.dot(&pos).powf(2.5);

	return -1.5 * h2 * pos / pos_fifth;
}

#[cfg(test)]
mod tests {
	use super::GRScene;

	use crate::{Camera, Sphere, Scene, Texture, TextureFiltering, TextureMode};
	use crate::raytrace::render::render;
	use image::{DynamicImage, Pixel, Rgb};
	use nalgebra::Vector3;

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
		let scene = GRScene(Scene {
			camera: Camera::new(30, 30, 10.0),
			sphere: Sphere {
				pos: Vector3::new(0.0, 0.0, -4.0),
				radius: 1.0,
				texture: Texture(img, TextureFiltering::Nearest, TextureMode::Clamp),
			},
			bgtex: None,
		}, 1.0, 10);
		render(scene, Some(&|p, msg| print!("[{}%] {}           \r", (1000.0 * p).round() / 10.0, msg)))
			.map(|i: DynamicImage| i.save("scene_gr.png"))
			.expect("saving file")
			.ok();
	}
}
