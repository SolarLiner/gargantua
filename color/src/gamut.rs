// Port of much of http://www.fourmilab.ch/documents/specrend/ - more precisely their C tool

use crate::color::Color;
use crate::xyz::XYZ;

use nalgebra::{Complex, Matrix3, Point2, Point3, Unit, Vector2, Vector3};

use std::f64;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[allow(dead_code)]
pub const ILLUMINANT_D65: XYChroma = XYChroma {
	x: 0.3127,
	y: 0.3291,
};
#[allow(dead_code)]
pub const ILLUMINANT_C: XYChroma = XYChroma {
	x: 0.3101,
	y: 0.3162,
};
#[allow(dead_code)]
pub const ILLUMINANT_E: XYChroma = XYChroma {
	x: 0.333333333,
	y: 0.333333333,
};

#[allow(dead_code)]
pub const SYSTEM_SRGB: ColorSystem = ColorSystem {
	red: XYChroma { x: 0.64, y: 0.32 },
	green: XYChroma { x: 0.3, y: 0.6 },
	blue: XYChroma { x: 0.15, y: 0.06 },
	white: ILLUMINANT_D65,
	gamma: 2.4,
};

#[allow(dead_code)]
pub const SYSTEM_CIERGB: ColorSystem = ColorSystem {
	red: XYChroma { x: 0.64, y: 0.32 },
	green: XYChroma { x: 0.3, y: 0.6 },
	blue: XYChroma { x: 0.15, y: 0.06 },
	white: ILLUMINANT_E,
	gamma: 2.4,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorSystem {
	pub red: XYChroma,
	pub green: XYChroma,
	pub blue: XYChroma,
	pub white: XYChroma,
	pub gamma: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct XYChroma {
	pub x: f64,
	pub y: f64,
}

impl ColorSystem {
	pub fn to_rgb(&self, xyz: &XYZ) -> Result<Color, &'static str> {
		self.get_matrix_to_xyz()
			.try_inverse()
			.ok_or("Couldn't inverse XYZ to RGB matrix")
			.map(|m| {
				let v: Vector3<f64> = xyz.clone().into();
				m * v
			})
			.map(|vec| self.gamma(&Color::with_system(vec.x, vec.y, vec.z, *self)))
	}

	pub fn to_xyz(&self, col: &Color) -> XYZ {
		let lin_col = self.gamma_inv(col);
		let m = self.get_matrix_to_xyz();
		let colvec: Vector3<f64> = lin_col.into();
		let xyzvec = m * colvec;

		XYZ {
			X: xyzvec.x,
			Y: xyzvec.y,
			Z: xyzvec.z,
		}
	}

	pub fn desaturate(&self, col: &Color, percent: f64) -> Result<Color, &'static str> {
		let (xy, Y) = self.to_xyz(col).to_chromaticity();

		self.to_rgb(&XYZ::chromaticity(
			XYChroma {
				x: lerp(percent, xy.x, self.white.x),
				y: lerp(percent, xy.y, self.white.y),
			},
			Y,
		))
	}

	pub fn gamma(&self, col: &Color) -> Color {
		let mut new_col = col.clone();
		new_col.red = gamma(self.gamma, col.red);
		new_col.green = gamma(self.gamma, col.green);
		new_col.blue = gamma(self.gamma, col.blue);

		return new_col;
	}

	pub fn gamma_inv(&self, col: &Color) -> Color {
		let mut new_col = col.clone();
		new_col.red = gamma_inv(self.gamma, col.red);
		new_col.green = gamma_inv(self.gamma, col.green);
		new_col.blue = gamma_inv(self.gamma, col.blue);

		return new_col;
	}

	fn get_matrix_to_xyz(&self) -> Matrix3<f64> {
		let red: Unit<Vector3<f64>> = self.red.into();
		let green: Unit<Vector3<f64>> = self.green.into();
		let blue: Unit<Vector3<f64>> = self.blue.into();

		Matrix3::from_columns(&[red.into_inner(), green.into_inner(), blue.into_inner()])
	}
}

impl fmt::Display for ColorSystem {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"ColorSystem(red={}, green={}, blue={}, white={}, gamma={})",
			self.red, self.green, self.blue, self.white, self.gamma
		)?;
		Ok(())
	}
}

impl Default for ColorSystem {
	fn default() -> Self {
		ColorSystem {
			red: XYChroma { x: 0.0, y: 0.0 },
			green: XYChroma { x: 1.0, y: 0.0 },
			blue: XYChroma { x: 1.0, y: 0.0 },
			white: ILLUMINANT_D65,
			gamma: 1.0,
		}
	}
}

impl Into<Matrix3<f64>> for ColorSystem {
	fn into(self) -> Matrix3<f64> {
		self.get_matrix_to_xyz()
	}
}

impl XYChroma {
	pub fn get_z(&self) -> f64 {
		return 1f64 - (self.x + self.y);
	}

	pub fn get_xyz(&self) -> (f64, f64, f64) {
		(self.x, self.y, self.get_z())
	}

	pub fn get_matrix_comp(&self) -> (f64, f64, f64) {
		(self.x / self.y, 1.0, self.get_z() / self.y)
	}
}

impl Add<XYChroma> for XYChroma {
	type Output = Self;

	fn add(self, rhs: XYChroma) -> Self {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Sub<XYChroma> for XYChroma {
	type Output = Self;

	fn sub(self, rhs: XYChroma) -> Self {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl AddAssign<XYChroma> for XYChroma {
	fn add_assign(&mut self, rhs: XYChroma) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl SubAssign<XYChroma> for XYChroma {
	fn sub_assign(&mut self, rhs: XYChroma) {
		self.x += rhs.x;
		self.y += rhs.y;
	}
}

impl From<Vector2<f64>> for XYChroma {
	fn from(val: Vector2<f64>) -> Self {
		Self { x: val.x, y: val.y }
	}
}

impl From<Unit<Vector3<f64>>> for XYChroma {
	fn from(val: Unit<Vector3<f64>>) -> Self {
		Self { x: val.x, y: val.y }
	}
}

impl From<Point2<f64>> for XYChroma {
	fn from(val: Point2<f64>) -> Self {
		Self::from(val.coords)
	}
}

impl From<Complex<f64>> for XYChroma {
	fn from(val: Complex<f64>) -> Self {
		Self {
			x: val.re,
			y: val.im,
		}
	}
}

impl Into<Vector2<f64>> for XYChroma {
	fn into(self) -> Vector2<f64> {
		Vector2::new(self.x, self.y)
	}
}

impl Into<Unit<Vector3<f64>>> for XYChroma {
	fn into(self) -> Unit<Vector3<f64>> {
		Unit::new_unchecked(Vector3::new(self.x, self.y, self.get_z()))
	}
}

impl Into<Complex<f64>> for XYChroma {
	fn into(self) -> Complex<f64> {
		Complex::new(self.x, self.y)
	}
}

impl fmt::Display for XYChroma {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "XYChroma(x={}, y={})", self.x, self.y)?;
		Ok(())
	}
}

fn lerp(x: f64, a: f64, b: f64) -> f64 {
	(1.0 - x) * a + x * b
}

fn gamma(exp: f64, value: f64) -> f64 {
	let alpha = 0.055;
	if value > 0.0031308 {
		return (1.0 + alpha) * value.powf(1.0 / exp) - alpha;
	} else {
		return 12.92 * value;
	}
}

fn gamma_inv(exp: f64, value: f64) -> f64 {
	let alpha = 0.055;
	if value > 0.04045 {
		return ((value + alpha) / (1.0 + alpha)).powf(exp);
	} else {
		return value / 12.92;
	}
}

#[cfg(test)]
mod tests {
	use crate::color::Color;
	use crate::gamut::{ColorSystem, XYChroma, ILLUMINANT_D65};
	use crate::xyz::XYZ;

	#[test]
	fn chroma_works() {
		let chroma = XYChroma { x: 0.0, y: 0.0 };
		assert_eq!(chroma.get_z(), 1.0);
		let chroma2 = XYChroma { x: 1.0, y: 0.0 };
		assert_eq!(chroma2.get_z(), 0.0);
	}

	#[test]
	fn colorsystem_works() {
		let red = XYChroma { x: 0.0, y: 0.0 };
		let green = XYChroma { x: 1.0, y: 0.0 };
		let blue = XYChroma { x: 0.0, y: 1.0 };
		let system = ColorSystem {
			red,
			green,
			blue,
			white: ILLUMINANT_D65,
			gamma: 0.0,
		};
		let white_point_xyz = XYZ::chromaticity(ILLUMINANT_D65, 1.0);
		let xyz_red = Color::with_system(1.0, 0.0, 0.0, system)
			.to_xyz(None)
			.expect("Couldn't convert to XYZ");
		let xyz_green = Color::with_system(0.0, 1.0, 0.0, system)
			.to_xyz(None)
			.expect("Couldn't convert to XYZ");

		println!("XYZ red: {:?}", xyz_red);
		println!("XYZ green: {:?}", xyz_green);
		assert_eq!(xyz_red.to_chromaticity().0, red);
		assert_eq!(xyz_green.to_chromaticity().0, green);
		assert_eq!(
			XYZ::chromaticity(blue, 1.0)
				.to_color(system)
				.expect("Couldn't convert to Color"),
			Color::new(0.0, 0.0, 1.0)
		);
		assert_eq!(
			system.to_rgb(&white_point_xyz).unwrap().normalize(),
			Color::new(1.0, 1.0, 1.0)
		);
	}
}
