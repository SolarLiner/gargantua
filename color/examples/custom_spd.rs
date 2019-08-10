use color::{XYZ, spectrum_to_xyz};

fn main() {
	let (X, Y, Z) = spectrum_to_xyz(&|y| if y > 560.0 && y < 620.0 { 5.0 } else { 0.0 });
	let xyz = XYZ {X,Y,Z};
	println!("Chromaticity/Luminance from spectrum: {:?}", xyz.to_chromaticity());
	let col = xyz.to_srgb().expect("Couldn't convert to sRGB").normalize().constrain();
	println!("Color from spectrum: {}\t{:?}", col.to_hex_code(false), col);
}
