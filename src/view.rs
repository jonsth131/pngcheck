use crate::png::scanline::Scanline;
use crate::png::{ColorType, Pixel, IHDR};
use image::{DynamicImage, GrayAlphaImage, GrayImage, RgbImage, RgbaImage};
use viuer::{print, Config};

pub fn view_image(scanlines: &[Scanline], ihdr: &IHDR) {
    let config = Config {
        transparent: ihdr.color_type.has_alpha(),
        width: Some(ihdr.width),
        height: Some(ihdr.height),
        ..Config::default()
    };

    println!(
        "Viewing image with width: {}, height: {}",
        ihdr.width, ihdr.height
    );

    let image = create_dynamic_image(scanlines, ihdr);
    print(&image, &config).unwrap();
}

fn create_dynamic_image(scanlines: &[Scanline], ihdr: &IHDR) -> DynamicImage {
    match ihdr.color_type {
        ColorType::TruecolorAlpha | ColorType::Indexed => {
            let mut img = RgbaImage::new(ihdr.width, ihdr.height);
            for (i, scanline) in scanlines.iter().enumerate() {
                for (j, pixel) in scanline.pixels.iter().enumerate() {
                    let (r, g, b, a) = match pixel {
                        Pixel::TruecolorAlpha(r, g, b, a) => (*r, *g, *b, *a),
                        _ => unreachable!(),
                    };
                    img.put_pixel(j as u32, i as u32, image::Rgba([r, g, b, a]));
                }
            }
            DynamicImage::ImageRgba8(img)
        }
        ColorType::Truecolor => {
            let mut img = RgbImage::new(ihdr.width, ihdr.height);
            for (i, scanline) in scanlines.iter().enumerate() {
                for (j, pixel) in scanline.pixels.iter().enumerate() {
                    let (r, g, b) = match pixel {
                        Pixel::Truecolor(r, g, b) => (*r, *g, *b),
                        _ => unreachable!(),
                    };
                    img.put_pixel(j as u32, i as u32, image::Rgb([r, g, b]));
                }
            }
            DynamicImage::ImageRgb8(img)
        }
        ColorType::Grayscale => {
            let mut img = GrayImage::new(ihdr.width, ihdr.height);
            for (i, scanline) in scanlines.iter().enumerate() {
                for (j, pixel) in scanline.pixels.iter().enumerate() {
                    let l = match pixel {
                        Pixel::Grayscale(l) => *l,
                        _ => unreachable!(),
                    };
                    img.put_pixel(j as u32, i as u32, image::Luma([l]));
                }
            }
            DynamicImage::ImageLuma8(img)
        }
        ColorType::GrayscaleAlpha => {
            let mut img = GrayAlphaImage::new(ihdr.width, ihdr.height);
            for (i, scanline) in scanlines.iter().enumerate() {
                for (j, pixel) in scanline.pixels.iter().enumerate() {
                    let (l, a) = match pixel {
                        Pixel::GrayscaleAlpha(l, a) => (*l, *a),
                        _ => unreachable!(),
                    };
                    img.put_pixel(j as u32, i as u32, image::LumaA([l, a]));
                }
            }
            DynamicImage::ImageLumaA8(img)
        }
    }
}
