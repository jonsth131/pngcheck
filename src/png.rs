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
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
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
}
