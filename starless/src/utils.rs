use color::Color;
use image::{Pixel, Rgba};
use nalgebra::Vector3;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use std::fmt::{Debug, Display, Formatter};

pub struct DimIterator<T> {
	width: T,
	height: T,
	x: T,
	y: T,
	sx: T,
	sy: T,
	started: bool,
	done: bool,
}

impl Iterator for DimIterator<u32> {
	type Item = (u32, u32);
	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}

		if self.width == 0 || self.height == 0 {
			self.done = true;
			return None;
		}

		if !self.started {
			self.started = true;
			return Some((self.sx, self.sy));
		}

		self.x += 1;
		if self.x >= self.width {
			self.x = 0;
			self.y += 1;
			if self.y >= self.height {
				self.done = true;
				return None;
			}
		}

		return Some((self.sx + self.x, self.sy + self.y));
	}
}

impl<T: Display> Display for DimIterator<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"({}, {}) of ({}, {})",
			self.x, self.y, self.width, self.height
		)
	}
}

impl<T: Debug> Debug for DimIterator<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(
			f,
			"DimIterator {{ x: {:?}, y: {:?}, width: {:?}, height: {:?} }}",
			self.x, self.y, self.width, self.height
		)
	}
}

impl<T: Default> DimIterator<T> {
	pub fn create(width: T, height: T, x: T, y: T) -> Self {
		Self {
			width,
			height,
			x: T::default(),
			y: T::default(),
			sx: x,
			sy: y,
			started: false,
			done: false,
		}
	}

	pub fn new(width: T, height: T) -> Self {
		Self {
			width,
			height,
			x: T::default(),
			y: T::default(),
			sx: T::default(),
			sy: T::default(),
			started: false,
			done: false,
		}
	}
}

pub fn rgba_to_color(col: Rgba<u8>) -> Color {
	let (red, green, blue, alpha) = col.channels4();

	return Color::from_u32(
		((alpha as u32) << 24 | (red as u32) << 16 | (green as u32) << 8 | blue as u32).into(),
	);
}

pub fn color_to_rgba(col: &Color) -> Rgba<u8> {
	Rgba::from_channels(
		(col.red * 255f64) as u8,
		(col.green * 255f64) as u8,
		(col.blue * 255f64) as u8,
		(col.alpha * 255f64) as u8,
	)
}

pub fn cartesian_to_spherical(vec: &Vector3<f64>) -> (f64, f64, f64) {
	let r = vec.dot(vec).sqrt();
	let phi = vec.y.atan2(vec.x);
	let theta = (vec.z / r).acos();

	(r, theta, phi)
}

#[cfg(test)]
mod tests {
	use crate::utils::DimIterator;

	#[test]
	fn dimiterator_works() {
		let it = DimIterator::new(2, 2);
		assert_eq!(
			it.collect::<Vec<(u32, u32)>>(),
			vec![(0, 0), (1, 0), (0, 1), (1, 1)]
		);
	}

	#[test]
	fn dimiterator_with_xy() {
		let it = DimIterator::create(2, 2, 3, 3);

		assert_eq!(
			it.collect::<Vec<(u32, u32)>>(),
			vec![(3, 3), (4, 3), (3, 4), (4, 4)]
		);
	}

	#[test]
	fn dimiterator_empty() {
		let mut it = DimIterator::new(0, 0);
		assert_eq!(it.next(), None);
	}
}
