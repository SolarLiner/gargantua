mod blackbody;
mod color;
mod gamut;
pub mod xyz;

pub use self::color::Color;
pub use blackbody::{blackbody_spectrum, spectrum_to_xyz};
pub use gamut::{ColorSystem, XYChroma};
pub use xyz::XYZ;

pub mod consts {
	pub use crate::blackbody::CIE_COLOR_MATCH;
	pub use crate::gamut::{ILLUMINANT_C, ILLUMINANT_D65, ILLUMINANT_E};
	pub use crate::gamut::{SYSTEM_CIERGB, SYSTEM_SRGB};

}
