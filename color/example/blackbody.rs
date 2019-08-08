use color::color::Color;

use color::gamut::SYSTEM_SRGB;
use color::xyz::XYZ;
use png::Encoder;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const WIDTH: u32 = 500;
const HEIGHT: u32 = 50;

fn main() {
    let colors: Vec<Color> = lin_space(5e3f64, 7e3f64, WIDTH as usize)
        .into_iter()
        .map(|x| {
            XYZ::blackbody(x)
                .to_color(SYSTEM_SRGB)
                .unwrap()
                // .constrain()
                .normalize()
        })
        .collect();
    for color in &colors {
        println!("{}", color);
    }
    let path = Path::new(r"output.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, WIDTH, HEIGHT);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);

    let mut png_data: Vec<u8> = Vec::new();
    let converted = png_convert(colors);
    for _ in 0..HEIGHT {
        for col in &converted {
            png_data.push(col.0);
            png_data.push(col.1);
            png_data.push(col.2);
        }
    }
    let res = encoder
        .write_header()
        .and_then(|mut wr| wr.write_image_data(png_data.as_slice()));
    match res {
        Ok(()) => println!("Done."),
        Err(err) => println!("ERROR: {}", err),
    };
}

fn lin_space(start: f64, end: f64, length: usize) -> Vec<f64> {
    let range = end - start;
    let step = range / (length as f64);
    let mut arr: Vec<f64> = Vec::new();

    for i in 0..length {
        arr.push(start + (i as f64) * step);
    }

    return arr;
}

fn png_convert(colors: Vec<Color>) -> Vec<(u8, u8, u8)> {
    colors.into_iter().map(color_to_u8).collect()
}

fn color_to_u8(color: Color) -> (u8, u8, u8) {
    (
        float_to_8bit(color.red),
        float_to_8bit(color.green),
        float_to_8bit(color.blue),
    )
}

fn float_to_8bit(x: f64) -> u8 {
    if x > 1.0 {
        return 255;
    } else if x < 0.0 {
        return 0;
    }
    return (x * 255f64) as u8;
}
