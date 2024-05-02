use flate2::read::ZlibDecoder;
use std::io::Read;

pub const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub const CRITICAL_CHUNKS: [&str; 4] = ["IHDR", "PLTE", "IDAT", "IEND"];

pub const ANCILLARY_CHUNKS: [&str; 18] = [
    "bKGD", "cHRM", "cICP", "dSIG", "eXIf", "gAMA", "hIST", "iCCP", "iTXt", "pHYs", "sBIT", "sPLT",
    "sRGB", "sTER", "tEXt", "tIME", "tRNS", "zTXt",
];

pub struct Chunk {
    pub length: u32,
    pub chunk_type: String,
    pub data: Option<Vec<u8>>,
    pub crc: u32,
}

impl Chunk {
    pub fn new(length: u32, chunk_type: String, data: Option<Vec<u8>>, crc: u32) -> Self {
        Self {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn calculate_checksum(&self) -> u32 {
        let crc32 = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        match &self.data {
            Some(data) => crc32.checksum(&[self.chunk_type.as_bytes(), &data[..]].concat()),
            None => crc32.checksum(self.chunk_type.as_bytes()),
        }
    }

    pub fn validate_checksum(&self) -> bool {
        self.calculate_checksum() == self.crc
    }
}

#[derive(Debug)]
pub enum Pixel {
    Grayscale(u8),
    Truecolor(u8, u8, u8),
    Indexed(u8),
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
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}

pub struct PLTE {
    pub entries: Vec<(u8, u8, u8)>,
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

    pub fn ihdr(&self) -> Option<IHDR> {
        for chunk in &self.chunks {
            if chunk.chunk_type == "IHDR" {
                let data = chunk.data.as_ref().unwrap();
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
        }

        None
    }

    pub fn plte(&self) -> Option<PLTE> {
        for chunk in &self.chunks {
            if chunk.chunk_type == "PLTE" {
                let data = chunk.data.as_ref().unwrap();
                let mut entries = vec![];
                for i in 0..data.len() / 3 {
                    entries.push((data[i * 3], data[i * 3 + 1], data[i * 3 + 2]));
                }
                return Some(PLTE { entries });
            }
        }

        None
    }

    pub fn get_pixel_data(&self) -> Result<Vec<Pixel>, std::io::Error> {
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
        let mut pixel_data = vec![];

        match ihdr.color_type {
            ColorType::Grayscale => {
                for pixel in idat_data {
                    pixel_data.push(Pixel::Grayscale(pixel));
                }
            }
            ColorType::Truecolor => {
                for i in 0..idat_data.len() / 3 {
                    pixel_data.push(Pixel::Truecolor(
                        idat_data[i * 3],
                        idat_data[i * 3 + 1],
                        idat_data[i * 3 + 2],
                    ));
                }
            }
            ColorType::Indexed => {
                for pixel in idat_data {
                    pixel_data.push(Pixel::Indexed(pixel));
                }
            }
            ColorType::GrayscaleAlpha => {
                for i in 0..idat_data.len() / 2 {
                    pixel_data.push(Pixel::GrayscaleAlpha(
                        idat_data[i * 2],
                        idat_data[i * 2 + 1],
                    ));
                }
            }
            ColorType::TruecolorAlpha => {
                for i in 0..idat_data.len() / 4 {
                    pixel_data.push(Pixel::TruecolorAlpha(
                        idat_data[i * 4],
                        idat_data[i * 4 + 1],
                        idat_data[i * 4 + 2],
                        idat_data[i * 4 + 3],
                    ));
                }
            }
        };

        Ok(pixel_data)
    }

    fn get_idat_data(&self) -> Vec<u8> {
        let mut idat_data = vec![];
        for chunk in &self.chunks {
            if chunk.chunk_type == "IDAT" {
                match &chunk.data {
                    Some(data) => idat_data.extend(data),
                    None => (),
                }
            }
        }
        idat_data
    }

    fn decompress_idat_data(&self) -> Result<Vec<u8>, std::io::Error> {
        let idat_data = self.get_idat_data();
        let mut zlib_decoder = ZlibDecoder::new(&idat_data[..]);
        let mut decompressed_data = vec![];
        let result = zlib_decoder.read_to_end(&mut decompressed_data);

        match result {
            Ok(_) => Ok(decompressed_data),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_checksum() {
        let chunk = Chunk::new(
            4,
            String::from("abcd"),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            283159080,
        );

        assert!(
            chunk.validate_checksum(),
            "Checksum failed, checksum was {}",
            chunk.calculate_checksum()
        );
    }

    #[test]
    fn test_validate_checksum_empty_data() {
        let chunk = Chunk::new(0, String::from("IEND"), None, 2923585666);

        assert!(
            chunk.validate_checksum(),
            "Checksum failed, checksum was {}",
            chunk.calculate_checksum()
        );
    }

    #[test]
    fn test_validate_invalid_checksum() {
        let chunk = Chunk::new(
            4,
            String::from("abcd"),
            Some(vec![0x01, 0x02, 0x03, 0x04]),
            11111111,
        );

        assert!(
            !chunk.validate_checksum(),
            "Checksum failed, checksum was {}",
            chunk.calculate_checksum()
        );
    }

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
