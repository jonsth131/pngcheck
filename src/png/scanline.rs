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

        let mut scanline_data = data
            .iter()
            .take(i + bytes_per_scanline)
            .skip(i + 1)
            .cloned()
            .collect::<Vec<u8>>();

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
    match ihdr.color_type {
        super::ColorType::Indexed => {
            let plte = plte.unwrap();
            let alpha = match plte.transparency.clone() {
                Some(transparency) => match transparency {
                    super::Transparency::Alpha(alpha) => alpha,
                    _ => panic!("Invalid transparency type"),
                },
                None => vec![],
            };

            return scanline
                .iter()
                .map(|&index| {
                    let (r, g, b) = plte.entries[index as usize];
                    let a = if index >= alpha.len() as u8 {
                        255
                    } else {
                        alpha[index as usize]
                    };
                    Pixel::TruecolorAlpha(r, g, b, a)
                })
                .collect();
        }
        super::ColorType::Grayscale => {
            return scanline
                .iter()
                .map(|&gray| Pixel::Grayscale(gray))
                .collect();
        }
        super::ColorType::GrayscaleAlpha => {
            return scanline
                .chunks(2)
                .map(|chunk| Pixel::GrayscaleAlpha(chunk[0], chunk[1]))
                .collect();
        }
        super::ColorType::Truecolor => {
            return scanline
                .chunks(3)
                .map(|chunk| Pixel::Truecolor(chunk[0], chunk[1], chunk[2]))
                .collect();
        }
        super::ColorType::TruecolorAlpha => {
            return scanline
                .chunks(4)
                .map(|chunk| Pixel::TruecolorAlpha(chunk[0], chunk[1], chunk[2], chunk[3]))
                .collect();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::ColorType;
    use crate::png::chunk::{CompressionMethod, FilterMethod, InterlaceMethod};

    #[test]
    fn test_parse_pixels() {
        let ihdr = IHDR {
            width: 1,
            height: 1,
            bit_depth: 8,
            color_type: ColorType::Grayscale,
            compression_method: CompressionMethod::Deflate,
            filter_method: FilterMethod::Adaptive,
            interlace_method: InterlaceMethod::None,
        };

        let plte = None;
        let scanline = vec![0x01];

        let pixels = parse_pixels(&ihdr, plte, &scanline);

        assert_eq!(pixels.len(), 1);
    }
}
