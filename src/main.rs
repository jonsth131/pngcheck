use clap::Parser;
use pngcheck::parse_file;
use pngcheck::png::{Chunk, Png};
use pngcheck::view::view_image;
use std::error::Error;

mod pretty_assert_printing;
mod tui;

//PNG check
#[derive(Parser)]
#[clap(author, about, version, long_about = None)]
enum Args {
    ///Check a PNG file
    Check {
        ///The PNG file to check
        file: String,
    },
    ///View a PNG file
    View {
        ///The PNG file to view
        file: String,
    },
    //Use a UI to view PNG information
    Ui {
        ///The PNG file to view
        file: String,
    },
}

fn print_banner() {
    println!();
    println!("██████╗ ███╗   ██╗ ██████╗      ██████╗██╗  ██╗███████╗ ██████╗██╗  ██╗");
    println!("██╔══██╗████╗  ██║██╔════╝     ██╔════╝██║  ██║██╔════╝██╔════╝██║ ██╔╝");
    println!("██████╔╝██╔██╗ ██║██║  ███╗    ██║     ███████║█████╗  ██║     █████╔╝ ");
    println!("██╔═══╝ ██║╚██╗██║██║   ██║    ██║     ██╔══██║██╔══╝  ██║     ██╔═██╗ ");
    println!("██║     ██║ ╚████║╚██████╔╝    ╚██████╗██║  ██║███████╗╚██████╗██║  ██╗");
    println!("╚═╝     ╚═╝  ╚═══╝ ╚═════╝      ╚═════╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝  ╚═╝");
    println!();
}

fn print_chunks(chunks: &Vec<Chunk>) {
    for chunk in chunks {
        println!("=============== {} ===============", chunk.chunk_type);
        println!("{}", chunk);
    }
}

fn read_file(file: &str) -> Result<Png, Box<dyn Error>> {
    let file = std::fs::File::open(file).expect("Failed to open file");

    parse_file(file)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args {
        Args::Check { file } => {
            print_banner();
            let data = read_file(&file)?;
            print_chunks(&data.chunks);
            println!("====================================");
            println!("Extra bytes: {:?}", data.extra_bytes);
        }
        Args::View { file } => {
            let data = read_file(&file)?;
            match data.ihdr() {
                Some(ihdr) => {
                    view_image(&data.get_scanlines()?, &ihdr);
                }
                None => eprintln!("IHDR chunk not found"),
            }
        }
        Args::Ui { file } => {
            let data = read_file(&file)?;
            tui::tui(&data)?;
        }
    };

    Ok(())
}
