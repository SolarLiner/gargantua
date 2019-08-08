mod blackbody;
pub mod color;
pub mod gamut;
pub mod xyz;

pub use color::Color;
pub use xyz::XYZ;
pub use gamut::{XYChroma, ColorSystem};

pub mod consts {
	pub use crate::gamut::{ILLUMINANT_C, ILLUMINANT_D65, ILLUMINANT_E};
	pub use crate::gamut::SYSTEM_SRGB;
}
