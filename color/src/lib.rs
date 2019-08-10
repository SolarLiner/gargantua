mod blackbody;
mod color;
mod gamut;
pub mod xyz;

pub use self::color::Color;
pub use xyz::XYZ;
pub use gamut::{ColorSystem, XYChroma};


pub mod consts {
	pub use crate::gamut::SYSTEM_SRGB;
	pub use crate::gamut::{ILLUMINANT_C, ILLUMINANT_D65, ILLUMINANT_E};
}
