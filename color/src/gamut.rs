// Port of much of http://www.fourmilab.ch/documents/specrend/ - more precisely their C tool

use crate::color::Color;
use crate::xyz::XYZ;

use nalgebra::{Matrix3, Vector3};

use std::f64;
use std::fmt;
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
		return match self
			.get_matrix()
			.and_then(|m| m.try_inverse().ok_or("Cannot inverse color system matrix"))
			.map(|m| m * Vector3::new(xyz.X, xyz.Y, xyz.Z))
		{
			Ok(vec) => Ok(self.gamma(&Color::with_system(vec[0], vec[1], vec[2], *self))),
			Err(err) => Err(err),
		};
	}

	pub fn to_xyz(&self, col: &Color) -> Result<XYZ, &'static str> {
		let lin_col = self.gamma_inv(col);
		let xyz_vec = self
			.get_matrix()
			.and_then(|m| Ok(m * Vector3::new(lin_col.red, lin_col.green, lin_col.blue)));
		return xyz_vec.map(|vec| XYZ {
			X: vec[0],
			Y: vec[1],
			Z: vec[2],
		});
	}

	pub fn desaturate(&self, col: &Color, percent: f64) -> Result<Color, &'static str> {
		let new_xy = match self.to_xyz(col).map(|xyz| xyz.to_chromaticity()) {
			Ok((xy, Y)) => Ok((
				XYChroma {
					x: lerp(percent, xy.x, self.white.x),
					y: lerp(percent, xy.y, self.white.y),
				},
				Y,
			)),
			Err(err) => Err(err),
		};
		return new_xy
			.map(|(xy, Y)| XYZ::chromaticity(xy, Y))
			.and_then(|xyz| xyz.to_color(*self));
	}

	fn gamma(&self, col: &Color) -> Color {
		let mut new_col = col.clone();
		new_col.red = gamma(self.gamma, col.red);
		new_col.green = gamma(self.gamma, col.green);
		new_col.blue = gamma(self.gamma, col.blue);

		return new_col;
	}

	fn gamma_inv(&self, col: &Color) -> Color {
		let mut new_col = col.clone();
		new_col.red = gamma_inv(self.gamma, col.red);
		new_col.green = gamma_inv(self.gamma, col.green);
		new_col.blue = gamma_inv(self.gamma, col.blue);

		return new_col;
	}

	fn get_matrix(&self) -> Result<Matrix3<f64>, &'static str> {
		let (xr, yr, zr) = self.red.get_matrix_comp();
		let (xg, yg, zg) = self.green.get_matrix_comp();
		let (xb, yb, zb) = self.blue.get_matrix_comp();

		let mat = Matrix3::new(xr, xg, xb, yr, yg, yb, zr, zg, zb);
		let ref_white = XYZ::chromaticity(self.white, 1.0);
		let sv: Option<Vector3<f64>> = mat
			.try_inverse()
			.map(|m| m * Vector3::new(ref_white.X, ref_white.Y, ref_white.Z));
		return match sv {
			Some(s) => {
				let mut mat2 = mat.clone();
				mat2[(0, 0)] *= s[0];
				mat2[(0, 1)] *= s[1];
				mat2[(0, 2)] *= s[2];
				mat2[(1, 0)] *= s[0];
				mat2[(1, 1)] *= s[1];
				mat2[(1, 2)] *= s[2];
				mat2[(2, 0)] *= s[0];
				mat2[(2, 1)] *= s[1];
				mat2[(2, 2)] *= s[2];
				Ok(mat2)
			}
			None => Err("Cannot transpose XYZ component matrix"),
		};
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
			red: XYChroma { x: 0.0, y: 0.001 },
			green: XYChroma { x: 1.0, y: 0.001 },
			blue: XYChroma { x: 1.0, y: 0.001 },
			white: ILLUMINANT_D65,
			gamma: 1.0,
		}
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
		assert_eq!(
			system.to_rgb(&XYZ::chromaticity(red, 1.0)).unwrap(),
			Color::new(1.0, 0.0, 0.0)
		);
		assert_eq!(
			system
				.to_rgb(&XYZ::chromaticity(green, 1.0))
				.unwrap()
				.normalize(),
			Color::new(0.0, 1.0, 0.0)
		);
		assert_eq!(
			system.to_rgb(&white_point_xyz).unwrap().normalize(),
			Color::new(1.0, 1.0, 1.0)
		);
	}
}
