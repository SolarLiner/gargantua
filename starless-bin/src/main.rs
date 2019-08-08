use image::{DynamicImage, Pixel, Rgb};
use nalgebra::Vector3;
use starless::{GRScene, Scene, Sphere, Texture, TextureFiltering, TextureMode};

type Vector = Vector3<f64>;

fn main() {
	let mut img = DynamicImage::new_rgb8(64, 64);
	for (x, y, p) in img.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
		*p = if x % 2 == 0 {
			Rgb::from_channels(255, 0, 100, 255)
		} else {
			Rgb::from_channels(100, 0, 255, 255)
		};
	}
	let scene = GRScene::new(Scene {
		width: 800,
		height: 600,
		fov: 90.0,
		sphere: Sphere {
			pos: Vector::new(0.0, 0.0, -25.0),
			radius: 1.0,
			texture: Texture(img, TextureFiltering::Nearest, TextureMode::Clamp),
		},
	});
	let report = &|p: f64, msg: String| print!("\r[{:2.1} %] {}               ", p * 100.0, msg);
	let start = std::time::Instant::now();
	// scene
	// 	.render(0.16, 250, Some(report))
	// 	.map(|i: DynamicImage| i.save("scene_gr.png"))
	// 	.expect("Error rendering image")
	// 	.ok();
	scene
		.render_threading(0.16, 500, Some(report))
		.map(|i| i.save("scene_gr_threading.png"))
		.expect("Error rendering image (threaded)")
		.ok();
	print!(
		"[100 %] Done in {:.2} s.         \n",
		start.elapsed().as_millis() as f64 / 1000.0
	);
}
