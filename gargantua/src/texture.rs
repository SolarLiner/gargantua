use color::consts::SYSTEM_SRGB;
use color::Color;
use image::{open, DynamicImage, GenericImageView, Pixel};
use nalgebra::Vector2;

use crate::utils::rgba_to_color;

#[derive(Clone, Debug)]
pub enum TextureMode {
	Clamp,
	Repeat,
	Transparent,
}
#[derive(Clone, Debug)]
pub enum TextureFiltering {
	Nearest,
	Bilinear,
}

#[derive(Clone)]
pub struct Texture(pub DynamicImage, pub TextureFiltering, pub TextureMode);

impl Texture {
	pub fn load(
		filepath: &str,
		filtering: TextureFiltering,
		mode: TextureMode,
	) -> image::ImageResult<Self> {
		return open(filepath).map(|b| Texture(b, filtering, mode));
	}

	pub fn uv(&self, uv: Vector2<f64>) -> Color {
		let (w, h) = self.0.dimensions();
		let (x, y) = (uv[0] * w as f64, uv[1] * h as f64);

		return match self.1 {
			TextureFiltering::Bilinear => self.bilinear(x, y),
			TextureFiltering::Nearest => self.get_pixel(x.round() as u32, y.round() as u32),
		};
	}

	pub fn get_pixel(&self, x: u32, y: u32) -> Color {
		let (width, height) = self.0.dimensions();
		return match self.2 {
			TextureMode::Clamp => rgba_to_color(
				self.0
					.get_pixel(clamp(x, 0, width - 1), clamp(y, 0, height - 1)),
			),
			TextureMode::Repeat => rgba_to_color(self.0.get_pixel(x % width, y % height)),
			TextureMode::Transparent => {
				let mut col = Color::default().into_with_system(SYSTEM_SRGB);
				col.alpha = 0.0;
				return col;
			}
		};
	}

	fn bilinear(&self, x: f64, y: f64) -> Color {
		let cx = x.floor();
		let cy = y.floor();
		let fx = x - cx;
		let fy = y - cy;

		let pix11 = self.get_pixel(cx as u32, cy as u32);
		let pix12 = self.get_pixel(cx as u32 + 1, cy as u32);
		let pix21 = self.get_pixel(cx as u32 + 1, cy as u32);
		let pix22 = self.get_pixel(cx as u32 + 1, cy as u32 + 1);
		let col_top = Color::mix(fx, pix11, pix12).unwrap();
		let col_bottom = Color::mix(fx, pix21, pix22).unwrap();
		return Color::mix(fy, col_top, col_bottom).unwrap();
	}
}

fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
	if x < min {
		return min;
	} else if x > max {
		return max;
	}
	return x;
}
