#![cfg_attr(feature="nightly", feature(plugin))]
#![cfg_attr(feature="nightly", plugin(clippy))]

extern crate image;
extern crate byteorder;
extern crate walkdir;
#[macro_use]
extern crate log;

pub mod pak;
pub mod lmp;
pub mod error;
pub mod palette;

pub use pak::{PakFile, create_pak};
pub use lmp::LmpImage;
pub use error::{QError, QResult};
pub use palette::{Palette, Color};
