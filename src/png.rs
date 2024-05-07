mod chunk;
mod compression;
mod filter;
pub mod scanline;

pub use crate::png::chunk::Chunk;
use crate::png::scanline::Scanline;

pub const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub const CRITICAL_CHUNKS: [&str; 4] = ["IHDR", "PLTE", "IDAT", "IEND"];

pub const ANCILLARY_CHUNKS: [&str; 18] = [
    "bKGD", "cHRM", "cICP", "dSIG", "eXIf", "gAMA", "hIST", "iCCP", "iTXt", "pHYs", "sBIT", "sPLT",
    "sRGB", "sTER", "tEXt", "tIME", "tRNS", "zTXt",
];

#[derive(Debug, Clone, Copy)]
pub enum Pixel {
    Grayscale(u8),
    Truecolor(u8, u8, u8),
    Indexed(u8, u8, u8),
    GrayscaleAlpha(u8, u8),
    TruecolorAlpha(u8, u8, u8, u8),
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    Truecolor,
    Indexed,
    GrayscaleAlpha,
    TruecolorAlpha,
}

#[derive(Debug)]
pub enum Transparency {
    Grey(u16),
    Rgb(u16, u16, u16),
    Alpha(Vec<u8>),
}

impl ColorType {
    pub fn has_alpha(&self) -> bool {
        matches!(self, ColorType::GrayscaleAlpha | ColorType::TruecolorAlpha)
    }
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

#[derive(Debug)]
pub struct PLTE {
    pub entries: Vec<(u8, u8, u8)>,
    pub transparency: Option<Transparency>,
}

#[derive(Debug)]
pub enum UnitSpecifier {
    Unknown,
    Meter,
}

#[derive(Debug)]
pub struct Phys {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit_specifier: UnitSpecifier,
}

#[derive(Debug)]
pub enum SrgbRenderingIntent {
    Perceptual,
    RelativeColorimetric,
    Saturation,
    AbsoluteColorimetric,
}

type gama = u32;

pub struct Png {
    pub chunks: Vec<Chunk>,
    pub extra_bytes: Option<Vec<u8>>,
}

impl Png {
    pub fn new(chunks: Vec<Chunk>, extra_bytes: Option<Vec<u8>>) -> Self {
        Self {
            chunks,
            extra_bytes,
        }
    }

    pub fn ihdr(&self) -> Option<IHDR> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "IHDR")?;

        if let Some(data) = &chunk.data {
            let color_type = match data[9] {
                0 => ColorType::Grayscale,
                2 => ColorType::Truecolor,
                3 => ColorType::Indexed,
                4 => ColorType::GrayscaleAlpha,
                6 => ColorType::TruecolorAlpha,
                _ => return None,
            };

            return Some(IHDR {
                width: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                height: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
                bit_depth: data[8],
                color_type,
                compression_method: data[10],
                filter_method: data[11],
                interlace_method: data[12],
            });
        }

        None
    }

    pub fn plte(&self) -> Option<PLTE> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "PLTE")?;

        if let Some(data) = &chunk.data {
            let mut entries = vec![];
            for i in 0..data.len() / 3 {
                entries.push((data[i * 3], data[i * 3 + 1], data[i * 3 + 2]));
            }
            return Some(PLTE {
                entries,
                transparency: self.trns(),
            });
        }

        None
    }

    pub fn trns(&self) -> Option<Transparency> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "tRNS")?;

        if let Some(data) = &chunk.data {
            match self.ihdr()?.color_type {
                ColorType::Grayscale => {
                    return Some(Transparency::Grey(u16::from_be_bytes([data[0], data[1]])))
                }
                ColorType::Truecolor => {
                    return Some(Transparency::Rgb(
                        u16::from_be_bytes([data[0], data[1]]),
                        u16::from_be_bytes([data[2], data[3]]),
                        u16::from_be_bytes([data[4], data[5]]),
                    ))
                }
                ColorType::Indexed => return Some(Transparency::Alpha(data.clone())),
                _ => (),
            }
        }

        None
    }

    pub fn phys(&self) -> Option<Phys> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "pHYs")?;

        if let Some(data) = &chunk.data {
            let unit_specifier = match data[8] {
                0 => UnitSpecifier::Unknown,
                1 => UnitSpecifier::Meter,
                _ => return None,
            };

            return Some(Phys {
                pixels_per_unit_x: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
                pixels_per_unit_y: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
                unit_specifier,
            });
        }

        None
    }

    pub fn srgb(&self) -> Option<SrgbRenderingIntent> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "sRGB")?;

        if let Some(data) = &chunk.data {
            match data[0] {
                0 => return Some(SrgbRenderingIntent::Perceptual),
                1 => return Some(SrgbRenderingIntent::RelativeColorimetric),
                2 => return Some(SrgbRenderingIntent::Saturation),
                3 => return Some(SrgbRenderingIntent::AbsoluteColorimetric),
                _ => (),
            }
        }

        None
    }

    pub fn gama(&self) -> Option<gama> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "gAMA")?;

        if let Some(data) = &chunk.data {
            return Some(u32::from_be_bytes([data[0], data[1], data[2], data[3]]));
        }

        None
    }

    fn get_idat_data(&self) -> Vec<u8> {
        self.chunks
            .iter()
            .filter(|c| c.chunk_type == "IDAT")
            .fold(vec![], |mut acc, c| {
                match &c.data {
                    Some(data) => acc.extend(data),
                    None => (),
                }
                acc
            })
    }

    fn decompress_idat_data(&self) -> Result<Vec<u8>, std::io::Error> {
        let idat_data = self.get_idat_data();
        compression::decompress(&idat_data)
    }

    pub fn get_scanlines(&self) -> Result<Vec<Scanline>, std::io::Error> {
        let ihdr = match self.ihdr() {
            Some(ihdr) => ihdr,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "IHDR chunk not found",
                ))
            }
        };

        let idat_data = self.decompress_idat_data()?;

        let scanlines = scanline::parse_scanlines(&ihdr, self.plte().as_ref(), &idat_data);

        Ok(scanlines)
    }
}

impl IHDR {
    pub fn bytes_per_pixel(&self) -> usize {
        match self.color_type {
            ColorType::Grayscale => 1,
            ColorType::Truecolor => 3,
            ColorType::Indexed => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::TruecolorAlpha => 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idat_data_single_idat_chunk() {
        let chunk = Chunk::new(
            4,
            String::from("IDAT"),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            283159080,
        );
        let png = Png::new(vec![chunk], None);

        assert_eq!(png.get_idat_data(), vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_get_idat_data_multiple_idat_chunks() {
        let chunk1 = Chunk::new(
            4,
            String::from("IDAT"),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            283159080,
        );
        let chunk2 = Chunk::new(
            4,
            String::from("IDAT"),
            Some(vec![0x05, 0x06, 0x07, 0x08]),
            283159080,
        );
        let png = Png::new(vec![chunk1, chunk2], None);

        assert_eq!(
            png.get_idat_data(),
            vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
    }

    #[test]
    fn test_get_idat_data_empty() {
        let png = Png::new(vec![], None);

        assert_eq!(png.get_idat_data(), vec![]);
    }

    #[test]
    fn test_decompress_idat_data() {
        let chunk1 = Chunk::new(
            4,
            String::from("IDAT"),
            Some(vec![
                0x78, 0x5e, 0x2b, 0x49, 0x2d, 0x2e, 0x01, 0x00, 0x04, 0x5d, 0x01, 0xc1,
            ]),
            283159080,
        );

        let png = Png::new(vec![chunk1], None);
        let decompressed_data = png.decompress_idat_data().unwrap();

        assert_eq!(decompressed_data, vec![0x74, 0x65, 0x73, 0x74]);
    }

    #[test]
    fn test_decompress_idat_data_invalid_stream() {
        let chunk1 = Chunk::new(
            4,
            String::from("IDAT"),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            283159080,
        );

        let png = Png::new(vec![chunk1], None);
        let result = png.decompress_idat_data().map_err(|e| e.kind());
        let expected = Err(std::io::ErrorKind::InvalidInput);

        assert_eq!(result, expected);
    }
}
