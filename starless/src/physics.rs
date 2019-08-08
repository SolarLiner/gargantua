use nalgebra::Vector3;
use crate::raytrace::Ray;

type Vector = Vector3<f64>;

#[derive(Clone, Debug)]
pub struct Particle {
	pos: Vector,
	vel: Vector,
	acc: Vector,
}

impl Particle {
	pub fn new(pos: Vector) -> Self {
		Self {
			pos,
			vel: Vector::zeros(),
			acc: Vector::zeros(),
		}
	}

	pub fn from_ray(r: &Ray) -> Self {
		Self {
			pos: r.origin,
			vel: r.direction,
			acc: Vector::zeros()
		}
	}

	pub fn add_force(&mut self, force: Vector) {
		self.acc += force;
	}

	pub fn update(&mut self, dt: f64) {
		self.vel += self.acc * dt;
		self.pos += self.vel * dt;
		self.acc = Vector::zeros();
	}

	pub fn pos(&self) -> Vector {
		self.pos
	}

	pub fn vel(&self) -> Vector {
		self.vel
	}

	pub fn acc(&self) -> Vector {
		self.acc
	}
}

impl Default for Particle {
	fn default() -> Self {
		Particle {
			pos: Vector::zeros(),
			vel: Vector::zeros(),
			acc: Vector::zeros(),
		}
	}
}
