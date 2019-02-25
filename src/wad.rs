use std::io;
use std::io::prelude::*;
use std::path::Path;

struct WadHeader {
    magic: &'static str,
}
