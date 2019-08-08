use crate::blackbody::{blackbody_spectrum, spectrum_to_xyz};
use crate::color::Color;
use crate::gamut::{ColorSystem, XYChroma, SYSTEM_SRGB};
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct XYZ {
	pub X: f64,
	pub Y: f64,
	pub Z: f64,
}

impl XYZ {
	pub fn chromaticity(xy: XYChroma, Y: f64) -> Self {
		let y_ratio = Y / xy.y;
		XYZ {
			X: xy.x * y_ratio,
			Y: Y,
			Z: xy.get_z() * y_ratio,
		}
	}
	pub fn from_spectral_data(f: &Fn(f64) -> f64) -> Self {
		let (X, Y, Z) = spectrum_to_xyz(f);

		XYZ { X, Y, Z }
	}
	pub fn blackbody(temperature: f64) -> Self {
		Self::from_spectral_data(&|y| blackbody_spectrum(temperature, y))
	}
}

impl XYZ {
	pub fn to_color(&self, system: ColorSystem) -> Result<Color, &'static str> {
		return system.to_rgb(self);
	}
	pub fn to_srgb(&self) -> Result<Color, &'static str> {
		SYSTEM_SRGB.to_rgb(self)
	}
	pub fn to_chromaticity(&self) -> (XYChroma, f64) {
		let sum = self.X + self.Y + self.Z;
		let chroma = XYChroma {
			x: self.X / sum,
			y: self.Y / sum,
		};
		return (chroma, self.Y);
	}
}

impl fmt::Display for XYZ {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "XYZ(X={}, Y={}, Z={})", self.X, self.Y, self.Z)?;
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::color::Color;
	use crate::gamut::SYSTEM_SRGB;

	#[test]
	fn conversion_to_xyz_and_back() {
		let col = Color::new(1.0, 1.0, 1.0);
		let converted = col
			.to_xyz(Some(SYSTEM_SRGB))
			.and_then(|xyz| xyz.to_color(SYSTEM_SRGB))
			.unwrap();
		assert_eq!(col, converted);
	}
}
