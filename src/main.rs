extern crate image;
extern crate byteorder;
extern crate walkdir;

pub mod pak;
pub mod lmp;
pub mod error;
pub mod palette;

use pak::{create_pak, PakFile};
use lmp::LmpImage;
use palette::Palette;
use std::{env, io};
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcommand: &str = &args[1];
    match subcommand {
        "create" => {
            let ref out_file = args[2];
            let ref folder = args[3];
            println!("Creating PAK file {} from folder {}", out_file, folder);
            let pak = create_pak(folder, &out_file).unwrap();
            println!("Result: {:?}", pak);
        },
        "extract" => {
            let ref in_file = args[2];
            let ref extract_path = args[3];
            println!("Extracting PAK file {} to {}", in_file, extract_path);
            let mut pak = PakFile::read(in_file).unwrap();
            pak.extract_to(extract_path).unwrap();
        },
        "convert_lmp" => {
            let ref in_file = args[2];
            let ref out_file = args[3];
            let ref palette_file = args[4];
            println!("Converting {} to {} with palette file {}!", in_file, out_file, palette_file);
            let mut reader = io::BufReader::new(File::open(in_file).unwrap());
            let lmp = LmpImage::read(&mut reader).unwrap();
            let mut reader = io::BufReader::new(File::open(palette_file).unwrap());
            let palette = Palette::read(&mut reader).unwrap();
            
            lmp.save_as(out_file, palette).unwrap();
        }
        x@_ => panic!("Unknown subcommand {}", x)
    }
}