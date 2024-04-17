use std::{env, io::BufReader, str};

use crate::{easy_br::EasyRead, pretty_assert_printing::soft_assert};

mod easy_br;
mod png;
mod pretty_assert_printing;

fn to_u32(bytes: &[u8]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();

    let mut buf = BufReader::new(std::fs::File::open(&args[1]).expect("Failed to open file"));

    let signature = buf.read_bytes(8)?;

    soft_assert("Signature", &signature, &png::HEADER);
    println!("Hello, world! Args: {:?}", args);

    let crc32 = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

    loop {
        let mut data = None;
        let length = buf.read_u32_be()?;
        let chunk_type = buf.read_bytes(4)?;
        let chunk_type = str::from_utf8(&chunk_type).unwrap().to_owned();
        if length != 0 {
            data = Some(buf.read_bytes(length as usize)?);
        }
        let crc = buf.read_u32_be()?;

        let chunk = png::Chunk {
            length,
            chunk_type,
            data,
            crc,
        };

        let checksum = match &chunk.data {
            Some(data) => crc32.checksum(&[chunk.chunk_type.as_bytes(), &data[..]].concat()),
            None => crc32.checksum(chunk.chunk_type.as_bytes()),
        };

        if png::CRITICAL_CHUNKS.contains(&&*chunk.chunk_type) {
            println!("Critical chunk: {:?}", chunk.chunk_type);
        } else if png::ANCILLARY_CHUNKS.contains(&&*chunk.chunk_type) {
            println!("Ancillary chunk: {:?}", chunk.chunk_type);
        } else {
            println!("Unknown chunk: {:?}", chunk.chunk_type);
        }

        println!("Chunk length: {:?}", chunk.length);
        println!("CRC: {:?}, Calculated CRC: {:?}", chunk.crc, checksum);

        if checksum != chunk.crc {
            println!("Checksum failed for chunk: {:?}", chunk.chunk_type);
        }

        if chunk.chunk_type == "IHDR" {
            let data = chunk.data.as_ref().unwrap();
            let ihdr = png::IHDR {
                width: to_u32(&data[0..4]),
                height: to_u32(&data[4..8]),
                bit_depth: data[8],
                color_type: data[9],
                compression_method: data[10],
                filter_method: data[11],
                interlace_method: data[12],
            };
            println!("{:?}", ihdr);
        }

        if chunk.chunk_type == "IEND" {
            break;
        }
    }

    Ok(())
}
