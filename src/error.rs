//! Defines error and result types for this library.

use std::io;
use walkdir;

/// Result of most things in this library that use IO or can fail for other reasons.
pub type QResult<T> = Result<T, QError>;

/// Error types.
#[derive(Debug)]
pub enum QError {
    /// See `std::io::Error`.
    IoError(io::Error),
    /// Invalid LMP file.
    InvalidLmp,
    /// Palette bigger than 256 colors
    InvalidPaletteSize,
    /// Wrong magic bytes in the file header.
    BadMagicBytes,
    /// Invalid filename for a PAK file.
    BadFileName,
    /// Unable to find the given file in the PAK.
    FileNotFound,
    /// See `walkdir::Error`.
    WalkDirError(walkdir::Error),
    /// Palette does not contain given color.
    ColorNotInPalette,
}

impl From<io::Error> for QError {
    fn from(err: io::Error) -> QError {
        QError::IoError(err)
    }
}

impl From<walkdir::Error> for QError {
    fn from(err: walkdir::Error) -> QError {
        QError::WalkDirError(err)
    }
}
