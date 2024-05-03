use flate2::read::ZlibDecoder;
use std::io::Read;

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut zlib_decoder = ZlibDecoder::new(data);
    let mut decompressed_data = vec![];
    let result = zlib_decoder.read_to_end(&mut decompressed_data);

    match result {
        Ok(_) => Ok(decompressed_data),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompress() {
        let data = vec![
            0x78, 0x5e, 0x2b, 0x49, 0x2d, 0x2e, 0x01, 0x00, 0x04, 0x5d, 0x01, 0xc1,
        ];
        let expected = vec![0x74, 0x65, 0x73, 0x74];

        let result = decompress(&data).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_decompress_invalid_stream() {
        let data = vec![0x00, 0x01, 0x02, 0x03];

        let result = decompress(&data).map_err(|e| e.kind());
        let expected = Err(std::io::ErrorKind::InvalidInput);

        assert_eq!(result, expected);
    }
}
