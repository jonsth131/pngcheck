use crate::png::filter::{filter_scanline, Filter};
use crate::png::{Pixel, IHDR, PLTE};

#[derive(Debug)]
pub struct Scanline {
    pub pixels: Vec<Pixel>,
}

pub fn parse_scanlines(ihdr: &IHDR, plte: Option<&PLTE>, data: &[u8]) -> Vec<Scanline> {
    let mut scanlines: Vec<Vec<u8>> = vec![];

    let bytes_per_pixel = ihdr.bytes_per_pixel();
    let bytes_per_scanline = 1 + ihdr.width as usize * bytes_per_pixel;

    for i in (0..data.len()).step_by(bytes_per_scanline) {
        let filter_type = match data[i] {
            0 => Filter::None,
            1 => Filter::Sub,
            2 => Filter::Up,
            3 => Filter::Average,
            4 => Filter::Paeth,
            _ => panic!("Invalid filter type"),
        };
        let mut scanline_data = vec![];
        for j in (i + 1)..(i + bytes_per_scanline) {
            scanline_data.push(data[j]);
        }

        let previous_scanline: Vec<u8> = match scanlines.last() {
            Some(scanline) => scanline.clone(),
            None => vec![0; bytes_per_scanline],
        };

        filter_scanline(
            filter_type,
            &previous_scanline,
            &mut scanline_data,
            bytes_per_pixel,
        );

        scanlines.push(scanline_data);
    }

    scanlines
        .iter()
        .map(|scanline| Scanline {
            pixels: parse_pixels(ihdr, plte, scanline),
        })
        .collect()
}

fn parse_pixels(ihdr: &IHDR, plte: Option<&PLTE>, scanline: &[u8]) -> Vec<Pixel> {
    let mut pixels = vec![];
    match ihdr.color_type {
        super::ColorType::Indexed => {
            let plte = plte.unwrap();
            for i in (0..scanline.len()).step_by(1) {
                let index = scanline[i];
                let (r, g, b) = plte.entries[index as usize];
                pixels.push(Pixel::Indexed(r, g, b));
            }
        }
        super::ColorType::Grayscale => {
            for i in (0..scanline.len()).step_by(1) {
                let gray = scanline[i];
                pixels.push(Pixel::Grayscale(gray));
            }
        }
        super::ColorType::Truecolor => {
            for i in (0..scanline.len()).step_by(3) {
                let r = scanline[i];
                let g = scanline[i + 1];
                let b = scanline[i + 2];
                pixels.push(Pixel::Truecolor(r, g, b));
            }
        }
        super::ColorType::GrayscaleAlpha => {
            for i in (0..scanline.len()).step_by(2) {
                let gray = scanline[i];
                let alpha = scanline[i + 1];
                pixels.push(Pixel::GrayscaleAlpha(gray, alpha));
            }
        }
        super::ColorType::TruecolorAlpha => {
            for i in (0..scanline.len()).step_by(4) {
                let r = scanline[i];
                let g = scanline[i + 1];
                let b = scanline[i + 2];
                let alpha = scanline[i + 3];
                pixels.push(Pixel::TruecolorAlpha(r, g, b, alpha));
            }
        }
    }

    pixels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::ColorType;

    #[test]
    fn test_parse_pixels() {
        let ihdr = IHDR {
            width: 1,
            height: 1,
            bit_depth: 8,
            color_type: ColorType::Grayscale,
            compression_method: 0,
            filter_method: 0,
            interlace_method: 0,
        };

        let plte = None;
        let scanline = vec![0x01];

        let pixels = parse_pixels(&ihdr, plte, &scanline);

        assert_eq!(pixels.len(), 1);
    }
}
