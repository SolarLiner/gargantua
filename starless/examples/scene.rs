use image::{DynamicImage, GenericImageView, Rgb, Pixel};
use nalgebra::{Vector3};
use starless::raytrace::{Sphere, Scene};
use starless::texture::{Texture, TextureFiltering, TextureMode};

fn main() {
	let mut texture = DynamicImage::new_rgb8(64, 64);
        for (x, y, p) in texture.as_mut_rgb8().unwrap().enumerate_pixels_mut() {
            if x % 2 == 0 {
                if y % 2 == 0 {
                    *p = Rgb::from_channels(255, 255, 255, 255);
                } else {
                    *p = Rgb::from_channels(0, 0, 0, 255);
                }
            } else {
                if y % 2 != 0 {
                    *p = Rgb::from_channels(255, 255, 255, 255);
                } else {
                    *p = Rgb::from_channels(0, 0, 0, 255);
                }
            }
        }
        let scn = Scene {
            width: 600,
            height: 380,
            fov: 45.0,
            sphere: Sphere {
                pos: Vector3::new(0.0, 0.0, -3.0),
                radius: 1.0,
                texture: Texture(texture, TextureFiltering::Nearest, TextureMode::Repeat),
            },
        };

        let img = scn.render();
        img.save("scene.png").unwrap();
        assert_eq!(scn.width, img.width());
        assert_eq!(scn.height, img.height());
}
