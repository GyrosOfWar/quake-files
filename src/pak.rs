//! Quake 1 PAK file. A PAK file is an archive format for storing
//! game content files.

#![allow(unused)]

use crate::error::*;
use byteorder::*;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{fmt, fs, io, str};
use walkdir::{DirEntry, WalkDir};

const DIR_ENTRY_SIZE: usize = 64;
const HEADER_SIZE: usize = 12;

#[derive(Debug)]
struct Header {
    magic: &'static [u8],
    dir_offset: i32,
    dir_length: i32,
}

impl Header {
    fn read<R>(reader: &mut R) -> QResult<Header>
    where
        R: io::Read,
    {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic);
        if &magic != b"PACK" {
            return Err(QError::BadMagicBytes);
        }
        let off = reader.read_i32::<LittleEndian>()?;
        let len = reader.read_i32::<LittleEndian>()?;

        Ok(Header {
            magic: b"PACK",
            dir_offset: off,
            dir_length: len,
        })
    }

    fn write<W>(&self, writer: &mut W) -> QResult<()>
    where
        W: io::Write,
    {
        writer.write_all(&self.magic)?;
        writer.write_i32::<LittleEndian>(self.dir_offset)?;
        writer.write_i32::<LittleEndian>(self.dir_length)?;

        Ok(())
    }
}

/// An entry in the directory of a PAK file. Each entry
/// has a name, a position (offset) from the beginning of the
/// file and a length.
struct DirectoryEntry {
    name: [u8; 56],
    position: i32,
    length: i32,
}

impl DirectoryEntry {
    fn read<R>(reader: &mut R) -> QResult<DirectoryEntry>
    where
        R: io::Read,
    {
        let mut name = [0; 56];
        reader.read_exact(&mut name)?;

        let pos = reader.read_i32::<LittleEndian>()?;
        let length = reader.read_i32::<LittleEndian>()?;

        Ok(DirectoryEntry {
            name: name,
            position: pos,
            length: length,
        })
    }

    /// The name of the file. Length is limited to 56 bytes.
    pub fn name_str(&self) -> &str {
        let name_bytes = &self.name;
        let nul = name_bytes.iter().position(|b| *b == 0).unwrap();
        let valid = &name_bytes[..nul];
        str::from_utf8(valid).unwrap()
    }

    fn write<W>(&self, writer: &mut W) -> QResult<()>
    where
        W: io::Write,
    {
        writer.write_all(&self.name)?;
        writer.write_i32::<LittleEndian>(self.position)?;
        writer.write_i32::<LittleEndian>(self.length)?;
        Ok(())
    }
}

impl fmt::Debug for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DirectoryEntry {{ name: {}, position: {}, length: {} }}",
            self.name_str(),
            self.position,
            self.length
        )
    }
}

/// A Quake 1 PAK file. Stores a list of directory entries
/// and allows reading single files or extracting the archive to a path.
#[derive(Debug)]
pub struct PakFile {
    name: String,
    directory: Vec<DirectoryEntry>,
    reader: io::BufReader<File>,
}

impl PakFile {
    /// Reads the .PAK file at the given path. Does not actually read
    /// any of the content files, just stores the directory entries.
    pub fn read<P>(path: P) -> QResult<PakFile>
    where
        P: AsRef<Path>,
    {
        let name = path
            .as_ref()
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(QError::BadFileName)?
            .into();
        let mut reader = io::BufReader::new(File::open(path)?);
        let header = Header::read(&mut reader)?;
        reader.seek(io::SeekFrom::Start(header.dir_offset as u64))?;
        let file_count = header.dir_length as usize / DIR_ENTRY_SIZE;

        let mut entries = vec![];
        for _ in 0..file_count {
            let entry = DirectoryEntry::read(&mut reader)?;
            entries.push(entry);
        }

        Ok(PakFile {
            name: name,
            directory: entries,
            reader: reader,
        })
    }

    /// Reads the contents of a file with the given name to a `Vec<u8>`.
    pub fn read_file(&mut self, name: &str) -> QResult<Vec<u8>> {
        let file = self.directory.iter().find(|f| f.name_str() == name);
        match file {
            Some(f) => {
                self.reader.seek(io::SeekFrom::Start(f.position as u64))?;
                let mut buf = vec![0; f.length as usize];
                let mut bytes_read = 0;
                while bytes_read < f.length {
                    bytes_read += self.reader.read(&mut buf)? as i32;
                }
                Ok(buf)
            }
            None => Err(QError::FileNotFound),
        }
    }

    /// Extracts the contents of this PAK file to the given location.
    pub fn extract_to<P>(&mut self, path: P) -> QResult<()>
    where
        P: AsRef<Path>,
    {
        // Screw you, borrowck!
        let names: Vec<String> = self.directory.iter().map(|f| f.name_str().into()).collect();
        for name in names {
            let content = self.read_file(&name)?;
            let full_path = path.as_ref().join(Path::new(&name));
            if let Some(p) = full_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            let mut file = File::create(full_path)?;
            file.write_all(&content)?;
        }
        Ok(())
    }
}

fn create_file_name(name: &OsStr) -> [u8; 56] {
    let mut buf = [0; 56];
    let s = name.to_string_lossy();
    for (i, ch) in s.chars().enumerate().take(56) {
        buf[i] = ch as u8;
    }

    buf
}

/// Creates a PAK file from the given directory tree and saves
/// it to the given filename.
pub fn create_pak<P>(base_dir: P, name: &str) -> QResult<PakFile>
where
    P: AsRef<Path>,
{
    let mut directory_offset = HEADER_SIZE as i32;
    let mut directory_length = 0;
    let mut writer = io::BufWriter::new(File::create(name)?);
    // Write magic bytes, then move ahead to the file section
    write!(writer, "PACK")?;
    writer.seek(io::SeekFrom::Start(HEADER_SIZE as u64))?;
    let mut directory_entries = vec![];

    for entry in WalkDir::new(base_dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let file_len = entry.metadata()?.len();
            let file_name = create_file_name(entry.file_name());

            let pak_entry = DirectoryEntry {
                name: file_name,
                length: file_len as i32,
                position: directory_offset,
            };

            directory_entries.push(pak_entry);

            let mut file = File::open(entry.path())?;
            // Read the file contents
            let mut buffer = vec![];
            file.read_to_end(&mut buffer)?;
            // Write the content to the PAK file
            writer.write_all(&buffer)?;

            // Move the offset and total length forward
            directory_offset += file_len as i32;
            directory_length += DIR_ENTRY_SIZE;
        }
    }

    // Write the directory entries
    for entry in &directory_entries {
        entry.write(&mut writer);
    }

    // Write the rest of the header
    writer.seek(io::SeekFrom::Start(4))?;
    writer.write_i32::<LittleEndian>(directory_offset)?;
    writer.write_i32::<LittleEndian>(directory_length as i32)?;

    // Assemble the struct
    let reader = io::BufReader::new(File::open(name)?);
    Ok(PakFile {
        name: name.into(),
        directory: directory_entries,
        reader: reader,
    })
}

#[cfg(test)]
mod tests {
    use super::PakFile;

    const PAK0: &str = "Id1/PAK0.PAK";

    #[test]
    fn read_pak() {
        let pak = PakFile::read(PAK0).unwrap();
        assert_eq!(pak.directory.len(), 339);
    }
}
