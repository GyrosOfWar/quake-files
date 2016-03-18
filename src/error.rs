use std::io;
use walkdir;

pub type QResult<T> = Result<T, QError>;

#[derive(Debug)]
pub enum QError {
    IoError(io::Error),
    InvalidLmp,
    InvalidPaletteSize,
    BadMagicBytes,
    BadFileName,
    FileNotFound,
    WalkDirError(walkdir::Error)
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
