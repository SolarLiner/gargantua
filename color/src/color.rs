
use crate::gamut::ColorSystem;
use crate::gamut::SYSTEM_SRGB;
use crate::xyz::XYZ;
use std::fmt;

/** Linear RGB Color structure */
#[derive(Clone, Debug, PartialEq)]
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
		println!(
			"DEBUG: color: {}, red: {}, green: {}, blue: {}",
			color, red, green, blue
		);

		return Color {
			red: red as f64 / 255f64,
			green: green as f64 / 255f64,
			blue: blue as f64 / 255f64,
			alpha: alpha as f64 / 255f64,
			system: Some(SYSTEM_SRGB),
		};
	}
	pub fn to_u32(&self) -> u32 {
		((self.alpha * 255f64) as u32) << 24
			| ((self.red * 255f64) as u32) << 16
			| ((self.green * 255f64) as u32) << 8
			| ((self.blue * 255f64) as u32)
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
		let max = self.red.max(self.green).max(self.blue);
		if max > 0f64 {
			return self.replace_rgb(self.red / max, self.green / max, self.blue / max);
		}
		return self.clone();
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
