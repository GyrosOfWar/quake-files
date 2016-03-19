extern crate image;
extern crate byteorder;
extern crate walkdir;

pub mod pak;
pub mod lmp;
pub mod error;
pub mod palette;

use pak::{create_pak, PakFile};
use std::str::from_utf8;
use std::env;

fn main() {
    // let pak = create_pak("test_files", "test.pak").unwrap();
    //println!("{:?}", pak);
    // let mut pak = PakFile::read("test.pak").unwrap();
    // println!("{:?}", pak);
    // let bytes = pak.read_file("test1.txt").unwrap();
    // println!("{:?}", from_utf8(&bytes));
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
        "read" => {
            let ref in_file = args[2];
            println!("Reading PAK file {}", in_file);
            let pak = PakFile::read(in_file).unwrap();
            println!("{:?}", pak);
        }
        x@_ => panic!("Unknown subcommand {}", x)
    }
}