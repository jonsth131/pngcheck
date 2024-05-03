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
