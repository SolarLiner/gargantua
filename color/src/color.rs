use crate::gamut::ColorSystem;
use crate::gamut::SYSTEM_SRGB;
use crate::xyz::XYZ;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/** Linear RGB Color structure */
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Color {
	pub red: f64,
	pub green: f64,
	pub blue: f64,
	pub alpha: f64,
	system: Option<ColorSystem>,
}

impl Color {
	pub fn new(r: f64, g: f64, b: f64) -> Self {
		Color {
			red: r,
			green: g,
			blue: b,
			alpha: 1.0,
			system: None,
		}
	}
	pub fn with_system(r: f64, g: f64, b: f64, system: ColorSystem) -> Self {
		Color {
			red: r,
			green: g,
			blue: b,
			alpha: 1.0,
			system: Some(system),
		}
	}
	pub fn from_u32(color: u32) -> Self {
		let alpha = (color >> 24 & 255) as u8;
		let red = (color >> 16 & 255) as u8;
		let green = (color >> 8 & 255) as u8;
		let blue = (color & 255) as u8;

		return Color {
			red: red as f64 / 255f64,
			green: green as f64 / 255f64,
			blue: blue as f64 / 255f64,
			alpha: alpha as f64 / 255f64,
			system: Some(SYSTEM_SRGB),
		};
	}
	pub fn to_u32(&self) -> u32 {
		((clamp(self.alpha, 0.0, 1.0) * 255f64) as u32) << 24
			| ((clamp(self.red, 0.0, 1.0) * 255f64) as u32) << 16
			| ((clamp(self.green, 0.0, 1.0) * 255f64) as u32) << 8
			| ((clamp(self.blue, 0.0, 1.0) * 255f64) as u32)
	}
	pub fn to_hex_code(&self, alpha: bool) -> String {
		if alpha {
			return format!("#{:x}", self.to_u32());
		} else {
			let hex_full = format!("{:x}", self.to_u32());
			return format!("#{}", &hex_full[2..8]);
		}
	}
	pub fn to_xyz(&self, default_system: Option<ColorSystem>) -> Result<XYZ, &'static str> {
		self.system
			.or(default_system)
			.ok_or("No color system provided, either from the Color object or as a default")
			.and_then(|s| s.to_xyz(self))
	}
	pub fn replace_rgb(&self, red: f64, green: f64, blue: f64) -> Self {
		Color {
			red,
			green,
			blue,
			alpha: self.alpha,
			system: self.system,
		}
	}
	pub fn in_gamut(&self) -> bool {
		(self.red >= 0f64) && (self.green >= 0f64) && (self.blue >= 0f64)
	}
	pub fn constrain(&self) -> Self {
		let w = -self.red.min(self.green).min(self.blue).min(0f64);
		if w > 0f64 {
			return self.replace_rgb(self.red + w, self.green + w, self.blue + w);
		}
		return self.clone();
	}
	pub fn normalize(&self) -> Self {
		let max = [self.red, self.green, self.blue]
			.iter()
			.fold(0.0f64, |p, c| if p.abs() < c.abs() { *c } else { p });
		return self.replace_rgb(self.red / max, self.green / max, self.blue / max);
	}
	pub fn set_system(&self, sys: ColorSystem) -> Self {
		let mut col = self.clone();
		col.system = Some(sys);

		return col;
	}
}

impl Color {
	pub fn mix(x: f64, a: Self, b: Self) -> Result<Self, &'static str> {
		if a.system != b.system {
			return Err("Cannot mix colors from different systems");
		}

		Ok(Color {
			red: lerp(x, a.red, b.red),
			green: lerp(x, a.green, b.green),
			blue: lerp(x, a.blue, b.blue),
			alpha: lerp(x, a.alpha, b.alpha),
			system: a.system,
		})
	}
}

impl Add<Color> for Color {
	type Output = Self;

	fn add(self, rhs: Self) -> Self {
		if self.system != rhs.system {
			panic!("Cannot add colors from different systems");
		}

		Self {
			red: self.red + rhs.red,
			green: self.green + rhs.green,
			blue: self.blue + rhs.blue,
			alpha: self.alpha + rhs.alpha,
			system: self.system,
		}
	}
}

impl Sub<Color> for Color {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		if self.system != rhs.system {
			panic!("Cannot subtract colors from different systems");
		}

		Self {
			red: self.red - rhs.red,
			green: self.green - rhs.green,
			blue: self.blue - rhs.blue,
			alpha: self.alpha - rhs.alpha,
			system: self.system,
		}
	}
}

impl AddAssign<Color> for Color {
	fn add_assign(&mut self, rhs: Self) {
		self.red += rhs.red;
		self.green += rhs.green;
		self.blue += rhs.blue;
		self.alpha += rhs.alpha;
	}
}

impl SubAssign<Color> for Color {
	fn sub_assign(&mut self, rhs: Self) {
		self.red -= rhs.red;
		self.green -= rhs.green;
		self.blue -= rhs.blue;
		self.alpha -= rhs.alpha;
	}
}

impl fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Color(red={}, green={}, blue={}, alpha={})",
			self.red, self.green, self.blue, self.alpha
		)?;
		return Ok(());
	}
}

#[cfg(test)]
mod tests {
	use crate::color::Color;
	use crate::gamut::SYSTEM_SRGB;
	use std::u32;

	#[test]
	fn it_works() {
		let col = Color::with_system(1.0, 1.0, 1.0, SYSTEM_SRGB);
		let col2 = Color::from_u32(u32::MAX);
		assert_eq!(col, col2);
	}

	#[test]
	fn shows_hex() {
		let col = Color::new(1.0, 0.0, 1.0);
		assert_eq!(
			String::from("#FF00FF"),
			col.to_hex_code(false).to_uppercase()
		);
		assert_eq!(
			String::from("#FFFF00FF"),
			col.to_hex_code(true).to_uppercase()
		);
	}
}

fn lerp(x: f64, a: f64, b: f64) -> f64 {
	(1.0 - x) * a + x * b
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
	if x < min {
		min
	} else if x > max {
		max
	} else {
		x
	}
}
