use image::{DynamicImage, Pixel, Rgb};
use nalgebra::{Translation3};

use gargantua::raytrace::render::render;
use gargantua::raytrace::{Camera, Scene, Sphere, Point};
use gargantua::texture::{Texture, TextureFiltering, TextureMode};

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
    let mut scn = Scene {
        camera: Camera::new(500, 500, 45.0),
        sphere: Sphere {
            pos: Point::new(0.0, 0.0, 0.0),
            radius: 1.0,
            texture: Texture(texture, TextureFiltering::Nearest, TextureMode::Repeat),
        },
        bgtex: None,
    };
    scn.set_camera(Some(Translation3::new(0.0, 0.0, 4.0)), None, None);

    render(scn, None)
        .expect("Couldn't render scene")
        .save("output_scene.png")
        .expect("Couldn't save image");
}
