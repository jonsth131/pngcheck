use crate::png::compression::decompress;

#[derive(Debug)]
pub enum ParsedChunk {
    IHDR(IHDR),
    PLTE(Vec<(u8, u8, u8)>),
    IDAT,
    IEND,
    Trns(Vec<u8>),
    Phys(Phys),
    Srgb(SrgbRenderingIntent),
    Gama(Gama),
    Bkgd(Bkgd),
    Sbit(Sbit),
    Itxt(Itxt),
    Text(Text),
    Ztxt(Ztxt),
    Unknown(String, Option<Vec<u8>>),
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: CompressionMethod,
    pub filter_method: FilterMethod,
    pub interlace_method: InterlaceMethod,
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

#[derive(Debug)]
pub enum ColorType {
    Grayscale,
    Truecolor,
    Indexed,
    GrayscaleAlpha,
    TruecolorAlpha,
}

impl ColorType {
    pub fn has_alpha(&self) -> bool {
        matches!(self, ColorType::GrayscaleAlpha | ColorType::TruecolorAlpha)
    }

    pub fn from(value: u8) -> Self {
        match value {
            0 => ColorType::Grayscale,
            2 => ColorType::Truecolor,
            3 => ColorType::Indexed,
            4 => ColorType::GrayscaleAlpha,
            6 => ColorType::TruecolorAlpha,
            _ => panic!("Invalid color type"),
        }
    }
}

#[derive(Debug)]
pub enum CompressionMethod {
    Deflate,
}

impl CompressionMethod {
    pub fn from(value: u8) -> Self {
        match value {
            0 => CompressionMethod::Deflate,
            _ => panic!("Invalid compression method"),
        }
    }
}

#[derive(Debug)]
pub enum FilterMethod {
    Adaptive,
}

impl FilterMethod {
    pub fn from(value: u8) -> Self {
        match value {
            0 => FilterMethod::Adaptive,
            _ => panic!("Invalid filter method"),
        }
    }
}

#[derive(Debug)]
pub enum InterlaceMethod {
    None,
    Adam7,
}

impl InterlaceMethod {
    pub fn from(value: u8) -> Self {
        match value {
            0 => InterlaceMethod::None,
            1 => InterlaceMethod::Adam7,
            _ => panic!("Invalid interlace method"),
        }
    }
}

#[derive(Debug)]
pub enum UnitSpecifier {
    Unknown,
    Meter,
}

impl UnitSpecifier {
    pub fn from(value: u8) -> Self {
        match value {
            0 => UnitSpecifier::Unknown,
            1 => UnitSpecifier::Meter,
            _ => panic!("Invalid unit specifier"),
        }
    }
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

impl SrgbRenderingIntent {
    pub fn from(value: u8) -> Self {
        match value {
            0 => SrgbRenderingIntent::Perceptual,
            1 => SrgbRenderingIntent::RelativeColorimetric,
            2 => SrgbRenderingIntent::Saturation,
            3 => SrgbRenderingIntent::AbsoluteColorimetric,
            _ => panic!("Invalid rendering intent"),
        }
    }
}

pub type Gama = u32;

#[derive(Debug)]
pub enum Bkgd {
    Grayscale(u16),
    Rgb(u16, u16, u16),
    Indexed(u8),
}

#[derive(Debug)]
pub enum Sbit {
    Grayscale(u8),
    Truecolor(u8, u8, u8),
    GrayscaleAlpha(u8, u8),
    TruecolorAlpha(u8, u8, u8, u8),
}

#[derive(Debug)]
pub struct Itxt {
    pub keyword: String,
    pub compression_flag: u8,
    pub compression_method: u8,
    pub language_tag: String,
    pub translated_keyword: String,
    pub text: String,
}

#[derive(Debug)]
pub struct Text {
    pub keyword: String,
    pub text: String,
}

#[derive(Debug)]
pub struct Ztxt {
    pub keyword: String,
    pub compression_method: u8,
    pub text: String,
}

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

    pub fn parse(&self) -> ParsedChunk {
        match self.chunk_type.as_str() {
            "IHDR" => ParsedChunk::IHDR(self.parse_ihdr()),
            "PLTE" => ParsedChunk::PLTE(self.parse_plte()),
            "IDAT" => ParsedChunk::IDAT,
            "IEND" => ParsedChunk::IEND,
            "tRNS" => ParsedChunk::Trns(self.data.as_ref().unwrap().to_vec()),
            "pHYs" => ParsedChunk::Phys(self.parse_phys()),
            "sRGB" => ParsedChunk::Srgb(self.parse_srgb()),
            "gAMA" => ParsedChunk::Gama(self.parse_gama()),
            "bKGD" => ParsedChunk::Bkgd(self.parse_bkgd()),
            "sBIT" => ParsedChunk::Sbit(self.parse_sbit()),
            "iTXt" => ParsedChunk::Itxt(self.parse_itxt()),
            "tEXt" => ParsedChunk::Text(self.parse_text()),
            "zTXt" => ParsedChunk::Ztxt(self.parse_ztxt()),
            _ => ParsedChunk::Unknown(self.chunk_type.clone(), self.data.clone()),
        }
    }

    fn parse_ihdr(&self) -> IHDR {
        let data = self.data.as_ref().unwrap();
        IHDR {
            width: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            height: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            bit_depth: data[8],
            color_type: ColorType::from(data[9]),
            compression_method: CompressionMethod::from(data[10]),
            filter_method: FilterMethod::from(data[11]),
            interlace_method: InterlaceMethod::from(data[12]),
        }
    }

    fn parse_plte(&self) -> Vec<(u8, u8, u8)> {
        let data = self.data.as_ref().unwrap();

        data.chunks_exact(3)
            .map(|chunk| (chunk[0], chunk[1], chunk[2]))
            .collect()
    }

    fn parse_phys(&self) -> Phys {
        let data = self.data.as_ref().unwrap();
        Phys {
            pixels_per_unit_x: u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
            pixels_per_unit_y: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            unit_specifier: UnitSpecifier::from(data[8]),
        }
    }

    fn parse_srgb(&self) -> SrgbRenderingIntent {
        let data = self.data.as_ref().unwrap();
        SrgbRenderingIntent::from(data[0])
    }

    fn parse_gama(&self) -> Gama {
        let data = self.data.as_ref().unwrap();
        u32::from_be_bytes([data[0], data[1], data[2], data[3]])
    }

    fn parse_bkgd(&self) -> Bkgd {
        let data = self.data.as_ref().unwrap();

        match data.len() {
            1 => Bkgd::Indexed(data[0]),
            2 => Bkgd::Grayscale(u16::from_be_bytes([data[0], data[1]])),
            6 => Bkgd::Rgb(
                u16::from_be_bytes([data[0], data[1]]),
                u16::from_be_bytes([data[2], data[3]]),
                u16::from_be_bytes([data[4], data[5]]),
            ),
            _ => panic!("Invalid bKGD chunk"),
        }
    }

    fn parse_sbit(&self) -> Sbit {
        let data = self.data.as_ref().unwrap();

        match data.len() {
            1 => Sbit::Grayscale(data[0]),
            3 => Sbit::Truecolor(data[0], data[1], data[2]),
            2 => Sbit::GrayscaleAlpha(data[0], data[1]),
            4 => Sbit::TruecolorAlpha(data[0], data[1], data[2], data[3]),
            _ => panic!("Invalid sBIT chunk"),
        }
    }

    fn parse_itxt(&self) -> Itxt {
        let data = self.data.as_ref().unwrap();

        let keyword_end = data.iter().position(|&x| x == 0).unwrap();
        let keyword = String::from_utf8(data[..keyword_end].to_vec()).unwrap();

        let compression_flag = data[keyword_end + 1];
        let compression_method = data[keyword_end + 2];

        let language_tag_start = keyword_end + 3;
        let language_tag_end = language_tag_start
            + data[language_tag_start..]
                .iter()
                .position(|&x| x == 0)
                .unwrap();

        let language_tag =
            String::from_utf8(data[language_tag_start..language_tag_end].to_vec()).unwrap();

        let translated_keyword_start = language_tag_end + 1;
        let translated_keyword_end = translated_keyword_start
            + data[translated_keyword_start..]
                .iter()
                .position(|&x| x == 0)
                .unwrap();

        let translated_keyword =
            String::from_utf8(data[translated_keyword_start..translated_keyword_end].to_vec())
                .unwrap();

        let text_start = translated_keyword_end + 1;
        let text = data[text_start..].to_vec();

        if compression_flag == 0 {
            Itxt {
                keyword,
                compression_flag,
                compression_method,
                language_tag,
                translated_keyword,
                text: String::from_utf8(text).unwrap(),
            }
        } else {
            let result = decompress(&text);

            match result {
                Ok(_) => Itxt {
                    keyword,
                    compression_flag,
                    compression_method,
                    language_tag,
                    translated_keyword,
                    text: String::from_utf8(result.unwrap()).unwrap(),
                },
                Err(e) => panic!("Error decompressing iTXt chunk: {}", e),
            }
        }
    }

    fn parse_text(&self) -> Text {
        let data = self.data.as_ref().unwrap();

        let keyword_end = data.iter().position(|&x| x == 0).unwrap();
        let keyword = String::from_utf8(data[..keyword_end].to_vec()).unwrap();

        let text = String::from_utf8(data[keyword_end + 1..].to_vec()).unwrap();

        Text { keyword, text }
    }

    fn parse_ztxt(&self) -> Ztxt {
        let data = self.data.as_ref().unwrap();

        let keyword_end = data.iter().position(|&x| x == 0).unwrap();
        let keyword = String::from_utf8(data[..keyword_end].to_vec()).unwrap();
        let compression_method = data[keyword_end + 1];

        let text = data[keyword_end + 2..].to_vec();

        let result = decompress(&text);

        match result {
            Ok(_) => Ztxt {
                keyword,
                compression_method,
                text: String::from_utf8(result.unwrap()).unwrap(),
            },
            Err(e) => panic!("Error decompressing zTXt chunk: {}", e),
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
}
