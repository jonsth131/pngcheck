mod chunk;
mod compression;
mod filter;
pub mod scanline;

pub use crate::png::chunk::{Chunk, ColorType, Gama, ParsedChunk, Phys, SrgbRenderingIntent, IHDR};
use crate::png::scanline::Scanline;

pub const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

#[derive(Debug, Clone, Copy)]
pub enum Pixel {
    Grayscale(u8),
    Truecolor(u8, u8, u8),
    GrayscaleAlpha(u8, u8),
    TruecolorAlpha(u8, u8, u8, u8),
}

#[derive(Debug, Clone)]
pub enum Transparency {
    Grey(u16),
    Rgb(u16, u16, u16),
    Alpha(Vec<u8>),
}

#[derive(Debug)]
pub struct PLTE {
    pub entries: Vec<(u8, u8, u8)>,
    pub transparency: Option<Transparency>,
}

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

    pub fn color_type(&self) -> ColorType {
        self.ihdr().map(|ihdr| ihdr.color_type).unwrap()
    }

    pub fn ihdr(&self) -> Option<IHDR> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "IHDR")?;

        match chunk.parse() {
            ParsedChunk::IHDR(ihdr) => Some(ihdr),
            _ => None,
        }
    }

    pub fn plte(&self) -> Option<PLTE> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "PLTE")?;

        match chunk.parse() {
            ParsedChunk::PLTE(entries) => Some(PLTE {
                entries,
                transparency: self.trns(),
            }),
            _ => None,
        }
    }

    pub fn trns(&self) -> Option<Transparency> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "tRNS")?;

        match chunk.parse() {
            ParsedChunk::Trns(data) => match self.ihdr()?.color_type {
                ColorType::Grayscale => {
                    Some(Transparency::Grey(u16::from_be_bytes([data[0], data[1]])))
                }
                ColorType::Truecolor => {
                    Some(Transparency::Rgb(
                        u16::from_be_bytes([data[0], data[1]]),
                        u16::from_be_bytes([data[2], data[3]]),
                        u16::from_be_bytes([data[4], data[5]]),
                    ))
                }
                ColorType::Indexed => Some(Transparency::Alpha(data.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn phys(&self) -> Option<Phys> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "pHYs")?;

        match chunk.parse() {
            ParsedChunk::Phys(phys) => Some(phys),
            _ => None,
        }
    }

    pub fn srgb(&self) -> Option<SrgbRenderingIntent> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "sRGB")?;

        match chunk.parse() {
            ParsedChunk::Srgb(intent) => Some(intent),
            _ => None,
        }
    }

    pub fn gama(&self) -> Option<Gama> {
        let chunk = self.chunks.iter().find(|c| c.chunk_type == "gAMA")?;

        match chunk.parse() {
            ParsedChunk::Gama(gama) => Some(gama),
            _ => None,
        }
    }

    pub fn get_pixels(&self) -> Result<Vec<Pixel>, std::io::Error> {
        let scanlines = self.get_scanlines()?;

        let pixels = scanlines.iter().fold(vec![], |mut acc, scanline| {
            acc.extend(scanline.pixels.iter().cloned());
            acc
        });

        Ok(pixels)
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

    fn decompress_idat_data(&self) -> Result<Vec<u8>, std::io::Error> {
        let idat_data = self.get_idat_data();
        compression::decompress(&idat_data)
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
