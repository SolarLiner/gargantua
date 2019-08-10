use color::{Color, XYZ};
use png::Encoder;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

const WIDTH: u32 = 2500;
const HEIGHT: u32 = 50;

fn main() {
    let colors = lin_space(1000.0, 25.0e3, WIDTH as usize)
        .into_iter()
        .map(|x| (x, XYZ::blackbody(x)));

    let img_path = Path::new(r"blackbody.png");
    let csv_path = Path::new(r"blackbody.csv");
    let img_file = File::create(img_path).expect("Couldn't create image file");
    let ref mut w = BufWriter::new(img_file);
    let mut encoder = Encoder::new(w, WIDTH, HEIGHT);
    let mut csv_writer = csv::WriterBuilder::default()
        .from_path(csv_path)
        .expect("Couldn't create CSV file");
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);

    csv_writer
        .write_record(&["Temp", "X", "Y", "Z", "x", "y", "R", "G", "B"])
        .expect("Couldn't write CSV headers");
    let mut png_data: Vec<u8> = Vec::new();
    for (t, xyz) in colors {
        let col = xyz
            .to_srgb()
            .expect("Couldn't convert XYZ to Color")
            .normalize();
        let (chroma, _) = xyz.to_chromaticity();
        let (r, g, b) = color_to_u8(col.clone());
        png_data.push(r);
        png_data.push(g);
        png_data.push(b);
        csv_writer
            .write_record(&[
                t.to_string(),
                xyz.X.to_string(),
                xyz.Y.to_string(),
                xyz.Z.to_string(),
                chroma.x.to_string(),
                chroma.y.to_string(),
                col.red.to_string(),
                col.green.to_string(),
                col.blue.to_string(),
            ])
            .expect("Couldn't add data to CSV");
    }
    let png_width_len = png_data.len();
    for _ in 1..HEIGHT {
        for i in 0..png_width_len {
            png_data.push(png_data[i]);
        }
    }
    let res = encoder
        .write_header()
        .and_then(|mut wr| wr.write_image_data(png_data.as_slice()));
    csv_writer.flush().expect("Couldn't write to CSV file");
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
