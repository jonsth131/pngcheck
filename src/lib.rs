use crate::easy_br::EasyRead;
use std::fs::File;
use std::io::{BufReader, Read};
use std::str;

mod easy_br;
pub mod png;

pub fn parse_file(file: File) -> Result<png::Png, Box<dyn std::error::Error>> {
    let mut buf = BufReader::new(file);

    let signature = buf.read_bytes(8)?;
    if signature != png::HEADER {
        return Err("Invalid PNG signature".into());
    }

    let mut chunks = vec![];

    loop {
        let mut data = None;
        let length = buf.read_u32_be()?;
        let chunk_type = buf.read_bytes(4)?;
        let chunk_type = str::from_utf8(&chunk_type)?;
        if length != 0 {
            data = Some(buf.read_bytes(length as usize)?);
        }
        let crc = buf.read_u32_be()?;

        chunks.push(png::Chunk::new(length, String::from(chunk_type), data, crc));

        if chunk_type == "IEND" {
            break;
        }
    }

    let mut extra_bytes: Vec<u8> = vec![];
    buf.read_to_end(&mut extra_bytes)?;

    if !extra_bytes.is_empty() {
        return Ok(png::Png::new(chunks, Some(extra_bytes)));
    }

    Ok(png::Png::new(chunks, None))
}
