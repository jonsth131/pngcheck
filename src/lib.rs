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
    let mut ihdr = None;

    loop {
        let mut data = None;
        let length = buf.read_u32_be()?;
        let chunk_type = buf.read_bytes(4)?;
        let chunk_type = str::from_utf8(&chunk_type)?;
        if length != 0 {
            data = Some(buf.read_bytes(length as usize)?);
        }
        let crc = buf.read_u32_be()?;

        if chunk_type == "IHDR" {
            let width = u32::from_be_bytes(data.as_ref().unwrap()[0..4].try_into()?);
            let height = u32::from_be_bytes(data.as_ref().unwrap()[4..8].try_into()?);
            let bit_depth = data.as_ref().unwrap()[8];
            let color_type = data.as_ref().unwrap()[9];
            let compression_method = data.as_ref().unwrap()[10];
            let filter_method = data.as_ref().unwrap()[11];
            let interlace_method = data.as_ref().unwrap()[12];

            ihdr = Some(png::IHDR {
                width,
                height,
                bit_depth,
                color_type,
                compression_method,
                filter_method,
                interlace_method,
            });
        }

        chunks.push(png::Chunk::new(length, String::from(chunk_type), data, crc));

        if chunk_type == "IEND" {
            break;
        }
    }

    let mut extra_bytes: Vec<u8> = vec![];
    buf.read_to_end(&mut extra_bytes)?;

    if !extra_bytes.is_empty() {
        return Ok(png::Png::new(ihdr, chunks, Some(extra_bytes)));
    }

    Ok(png::Png::new(ihdr, chunks, None))
}
