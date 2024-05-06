use pngcheck::parse_file;
use pngcheck::png::Chunk;
use pngcheck::view::view_image;
use std::env;

mod pretty_assert_printing;

fn print_chunks(chunks: &Vec<Chunk>) {
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

    let parsed_png = parse_file(file);

    match parsed_png {
        Ok(data) => {
            println!("IHDR: {:?}", data.ihdr());
            print_chunks(&data.chunks);
            println!("Extra bytes: {:?}", data.extra_bytes);

            match data.ihdr() {
                Some(ihdr) => {
                    view_image(&data.get_scanlines()?, &ihdr);
                }
                None => println!("IHDR chunk not found"),
            }
        }
        Err(e) => println!("Error parsing file: {:?}", e),
    }

    Ok(())
}
