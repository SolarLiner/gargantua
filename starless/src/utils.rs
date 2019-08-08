use color::Color;
use image::{Pixel, Rgba};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use std::fmt::{Debug, Display, Formatter};

pub struct DimIterator<T> {
	width: T,
	height: T,
	x: T,
	y: T,
	done: bool,
}

impl Iterator for DimIterator<u32> {
	type Item = (u32, u32);
	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			return None;
		}
		let res = Some((self.x, self.y));

		if self.x + 1 == self.width {
			if self.y + 1 == self.height {
				self.done = true;
				return res;
			}
			self.y += 1;
			self.x = 0;
		} else {
			self.x += 1;
		}

		return res;
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

impl<T> DimIterator<T> {
	pub fn create(width: T, height: T, x: T, y: T) -> Self {
		Self {
			width,
			height,
			x,
			y,
			done: false,
		}
	}
}

impl<T: Default> DimIterator<T> {
	pub fn new(width: T, height: T) -> Self {
		Self {
			width,
			height,
			x: T::default(),
			y: T::default(),
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
}
