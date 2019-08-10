use crate::raytrace::{Point, Vector, Ray};

#[derive(Clone, Debug)]
pub struct Particle {
	pos: Point,
	vel: Vector,
	acc: Vector,
}

impl Particle {
	pub fn new(pos: Point) -> Self {
		Self {
			pos,
			vel: Vector::zeros(),
			acc: Vector::zeros(),
		}
	}

	pub fn from_ray(r: &Ray) -> Self {
		Self {
			pos: r.origin,
			vel: r.direction.as_ref().clone(),
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

	pub fn pos(&self) -> Point {
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
			pos: pt_zero(),
			vel: Vector::zeros(),
			acc: Vector::zeros(),
		}
	}
}

fn pt_zero() -> Point {
	Point::new(0.0, 0.0, 0.0)
}
