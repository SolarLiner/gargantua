use crate::blackbody::{blackbody_spectrum, spectrum_to_xyz};
use crate::color::Color;
use crate::gamut::{ColorSystem, XYChroma, SYSTEM_SRGB};
use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use nalgebra::{Point3, Vector3};

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

impl Add<XYZ> for XYZ {
	type Output = Self;
	fn add(self, rhs: XYZ) -> Self {
		Self {
			X: self.X + rhs.X,
			Y: self.Y + rhs.Y,
			Z: self.Z + rhs.Z,
		}
	}
}

impl AddAssign<XYZ> for XYZ {
	fn add_assign(&mut self, rhs: XYZ) {
		self.X += rhs.X;
		self.Y += rhs.Y;
		self.Z += rhs.Z;
	}
}

impl Sub<XYZ> for XYZ {
	type Output = Self;
	fn sub(self, rhs: XYZ) -> Self {
		Self {
			X: self.X - rhs.X,
			Y: self.Y - rhs.Y,
			Z: self.Z - rhs.Z,
		}
	}
}

impl SubAssign<XYZ> for XYZ {
	fn sub_assign(&mut self, rhs: XYZ) {
		self.X -= rhs.X;
		self.Y -= rhs.Y;
		self.Z -= rhs.Z;
	}
}

impl Mul<f64> for XYZ {
	type Output = Self;
	fn mul(self, rhs: f64) -> Self {
		Self {
			X: self.X * rhs,
			Y: self.Y * rhs,
			Z: self.Z * rhs,
		}
	}
}

impl MulAssign<f64> for XYZ {
	fn mul_assign(&mut self, rhs: f64) {
		self.X *= rhs;
		self.Y *= rhs;
		self.Z *= rhs;
	}
}

impl Div<f64> for XYZ {
	type Output = Self;
	fn div(self, rhs: f64) -> Self {
		Self {
			X: self.X / rhs,
			Y: self.Y / rhs,
			Z: self.Z / rhs,
		}
	}
}

impl DivAssign<f64> for XYZ {
	fn div_assign(&mut self, rhs: f64) {
		self.X *= rhs;
		self.Y *= rhs;
		self.Z *= rhs;
	}
}

impl From<[f64; 3]> for XYZ {
	fn from(val: [f64; 3]) -> Self {
		Self {
			X: val[0],
			Y: val[1],
			Z: val[2],
		}
	}
}

impl From<Vector3<f64>> for XYZ {
	fn from(val: Vector3<f64>) -> Self {
		XYZ {
			X: val.x,
			Y: val.y,
			Z: val.z,
		}
	}
}

impl From<Point3<f64>> for XYZ {
	fn from(val: Point3<f64>) -> Self {
		XYZ::from(val.coords)
	}
}

impl Into<[f64; 3]> for XYZ {
	fn into(self) -> [f64; 3] {
		[self.X, self.Y, self.Z]
	}
}

impl Into<Vector3<f64>> for XYZ {
	fn into(self) -> Vector3<f64> {
		Vector3::new(self.X, self.Y, self.Z)
	}
}

impl Into<Point3<f64>> for XYZ {
	fn into(self) -> Point3<f64> {
		Point3::new(self.X, self.Y, self.Z)
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
