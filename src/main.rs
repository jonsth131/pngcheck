use std::{env, io::BufReader};

use crate::{easy_br::EasyRead, pretty_assert_printing::soft_assert};

mod easy_br;
mod pretty_assert_printing;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = env::args().collect();

    let mut buf = BufReader::new(std::fs::File::open(&args[1]).expect("Failed to open file"));

    let signature = buf.read_bytes(8)?;

    soft_assert(
        "Signature",
        &signature,
        &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
    );

    println!("Hello, world! Args: {:?}", args);

    Ok(())
}
