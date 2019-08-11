use crate::gamut::ColorSystem;
use crate::gamut::SYSTEM_SRGB;
use crate::xyz::XYZ;
use nalgebra::{Vector3, Vector4};
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

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

		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);

			s.gamma(&Self {
				red: col.red + rhs.red,
				green: col.green + rhs.green,
				blue: col.blue + rhs.blue,
				alpha: col.alpha + rhs.alpha,
				system: self.system,
			})
		} else {
			Self {
				red: self.red + rhs.red,
				green: self.green + rhs.green,
				blue: self.blue + rhs.blue,
				alpha: self.alpha + rhs.alpha,
				system: self.system,
			}
		}
	}
}

impl Sub<Color> for Color {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self {
		if self.system != rhs.system {
			panic!("Cannot subtract colors from different systems");
		}

		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);
			let rhs = s.gamma_inv(&rhs);

			s.gamma(&Self {
				red: col.red - rhs.red,
				green: col.green - rhs.green,
				blue: col.blue - rhs.blue,
				alpha: col.alpha - rhs.alpha,
				system: self.system,
			})
		} else {
			Self {
				red: self.red - rhs.red,
				green: self.green - rhs.green,
				blue: self.blue - rhs.blue,
				alpha: self.alpha - rhs.alpha,
				system: self.system,
			}
		}
	}
}

impl AddAssign<Color> for Color {
	fn add_assign(&mut self, rhs: Self) {
		let col = self.clone() + rhs;
		self.red = col.red;
		self.green = col.green;
		self.blue = col.blue;
		self.alpha = col.alpha;
	}
}

impl SubAssign<Color> for Color {
	fn sub_assign(&mut self, rhs: Self) {
		let col = self.clone() - rhs;
		self.red = col.red;
		self.green = col.green;
		self.blue = col.blue;
		self.alpha = col.alpha;
	}
}

impl Mul<f64> for Color {
	type Output = Self;
	fn mul(self, rhs: f64) -> Self {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);

			s.gamma(&Self {
				red: col.red * rhs,
				green: col.green * rhs,
				blue: col.blue * rhs,
				alpha: col.alpha * rhs,
				system: self.system,
			})
		} else {
			Self {
				red: self.red * rhs,
				green: self.green * rhs,
				blue: self.blue * rhs,
				alpha: self.alpha * rhs,
				system: self.system,
			}
		}
	}
}

impl Div<f64> for Color {
	type Output = Self;
	fn div(self, rhs: f64) -> Self {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);

			s.gamma(&Self {
				red: col.red / rhs,
				green: col.green / rhs,
				blue: col.blue / rhs,
				alpha: col.alpha / rhs,
				system: self.system,
			})
		} else {
			Self {
				red: self.red / rhs,
				green: self.green / rhs,
				blue: self.blue / rhs,
				alpha: self.alpha / rhs,
				system: self.system,
			}
		}
	}
}

impl MulAssign<f64> for Color {
	fn mul_assign(&mut self, rhs: f64) {
		let col = self.clone() * rhs;
		self.red = col.red;
		self.green = col.green;
		self.blue = col.blue;
		self.alpha = col.alpha;
	}
}

impl DivAssign<f64> for Color {
	fn div_assign(&mut self, rhs: f64) {
		let col = self.clone() / rhs;
		self.red = col.red;
		self.green = col.green;
		self.blue = col.blue;
		self.alpha = col.alpha;
	}
}

impl From<u32> for Color {
	fn from(val: u32) -> Self {
		Self::from_u32(val)
	}
}

impl From<(f64, f64, f64, f64)> for Color {
	fn from(val: (f64, f64, f64, f64)) -> Self {
		let mut col = Color::new(val.0, val.1, val.2);
		col.alpha = val.3;

		return col;
	}
}

impl From<[f64; 3]> for Color {
	fn from(val: [f64; 3]) -> Self {
		Color::new(val[0], val[1], val[2])
	}
}

impl From<[f64; 4]> for Color {
	fn from(val: [f64; 4]) -> Self {
		let mut col = Color::new(val[0], val[1], val[2]);
		col.alpha = val[3];

		return col;
	}
}

impl From<XYZ> for Color {
	fn from(val: XYZ) -> Self {
		val.to_color(crate::consts::SYSTEM_CIERGB)
			.expect("Couldn't convert to color")
	}
}

impl From<Vector3<f64>> for Color {
	fn from(val: Vector3<f64>) -> Color {
		Color::new(val.x, val.y, val.z)
	}
}

impl From<Vector4<f64>> for Color {
	fn from(val: Vector4<f64>) -> Color {
		let mut col = Color::new(val.x, val.y, val.z);
		col.alpha = val.w;

		return col;
	}
}

impl Into<u32> for Color {
	fn into(self) -> u32 {
		self.to_u32()
	}
}

impl Into<(f64, f64, f64, f64)> for Color {
	fn into(self) -> (f64, f64, f64, f64) {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);
			(col.red, col.green, col.blue, col.alpha)
		} else {
			(self.red, self.green, self.blue, self.alpha)
		}
	}
}

impl Into<[f64; 3]> for Color {
	fn into(self) -> [f64; 3] {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);

			[col.red, col.green, col.blue]
		} else {
			[self.red, self.green, self.blue]
		}
	}
}

impl Into<[f64; 4]> for Color {
	fn into(self) -> [f64; 4] {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);
			[col.red, col.green, col.blue, col.alpha]
		} else {
			[self.red, self.green, self.blue, self.alpha]
		}
	}
}

impl Into<Vector3<f64>> for Color {
	fn into(self) -> Vector3<f64> {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);
			Vector3::new(col.red, col.green, col.blue)
		} else {
			Vector3::new(self.red, self.green, self.blue)
		}
	}
}

impl Into<Vector4<f64>> for Color {
	fn into(self) -> Vector4<f64> {
		if let Some(s) = self.system {
			let col = s.gamma_inv(&self);

			Vector4::new(col.red, col.green, col.blue, col.alpha)
		} else {
			Vector4::new(self.red, self.green, self.blue, self.alpha)
		}
	}
}

impl fmt::Display for Color {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"rgba({:.1}, {:.1}, {:.1}, {:.1})",
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

	#[test]
	fn impl_std_traits() {
		let input: u32 = 0xFF2340;
		let output: u32 = Color::from(input).into();
		assert_eq!(output, input);
	}

	#[test]
	fn can_constrain_into_gamut() {
		let col = Color::new(0.0, 0.5, -2.5);
		let constrained = col.constrain();
		assert!(constrained.in_gamut());
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
