use pngcheck::parse_file;
use pngcheck::png::Chunk;
use std::env;

mod pretty_assert_printing;

fn print_chunks(chunks: Vec<Chunk>) {
    for chunk in chunks {
        println!("Chunk type: {:?}", chunk.chunk_type);
        println!("Chunk length: {:?}", chunk.length);
        println!(
            "CRC: {:?}, Valid: {:?}",
            chunk.crc,
            chunk.validate_checksum()
        );
    }
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();

    let file = std::fs::File::open(&args[1]).expect("Failed to open file");

    let parsed_file = parse_file(file);

    match parsed_file {
        Ok(data) => print_chunks(data),
        Err(e) => println!("Error parsing file: {:?}", e),
    }

    Ok(())
}
