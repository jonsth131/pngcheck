use std::collections::HashMap;

use crate::png::Pixel;
use crate::png::Png;

pub fn analyze(png: &Png) -> Result<HashMap<String, Vec<u8>>, Box<dyn std::error::Error>> {
    let pixels = png.get_pixels()?.to_vec();

    match png.color_type() {
        crate::png::ColorType::Grayscale => {
            println!("Grayscale color type not supported");
        }
        crate::png::ColorType::Truecolor => {
            return Ok(analyze_truecolor(&pixels));
        }
        crate::png::ColorType::GrayscaleAlpha => {
            println!("GrayscaleAlpha color type not supported");
        }
        crate::png::ColorType::TruecolorAlpha => {
            return Ok(analyze_truecolor_alpha(&pixels));
        }
        crate::png::ColorType::Indexed => {
            panic!("Indexed color type not supported");
        }
    }

    Ok(HashMap::new())
}

fn analyze_truecolor(pixels: &Vec<Pixel>) -> HashMap<String, Vec<u8>> {
    let mut rgb_values: HashMap<String, Vec<u8>> = HashMap::new();

    pixels.iter().for_each(|pixel| match pixel {
        Pixel::Truecolor(r, g, b) | Pixel::TruecolorAlpha(r, g, b, _) => {
            let key = format!("r");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(*r);

            let key = format!("g");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(*g);

            let key = format!("b");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(*b);

            let key = format!("r+g");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(r.wrapping_add(*g));

            let key = format!("r+b");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(r.wrapping_add(*b));

            let key = format!("g+b");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(g.wrapping_add(*b));

            let key = format!("r+g+b");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(r.wrapping_add(*g).wrapping_add(*b));

            let key = format!("rgb");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(*r);
            count.push(*g);
            count.push(*b);
        }
        _ => {}
    });

    rgb_values
}

fn analyze_truecolor_alpha(pixels: &Vec<Pixel>) -> HashMap<String, Vec<u8>> {
    let mut rgb_values: HashMap<String, Vec<u8>> = analyze_truecolor(pixels);

    pixels.iter().for_each(|pixel| match pixel {
        Pixel::TruecolorAlpha(r, g, b, a) => {
            let key = format!("a");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(*a);

            let key = format!("r+g+b+a");
            let count = rgb_values.entry(key).or_insert(vec![]);
            count.push(r.wrapping_add(*g).wrapping_add(*b).wrapping_add(*a));
        }
        _ => {}
    });

    rgb_values
}
