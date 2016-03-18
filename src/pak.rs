#![allow(unused)]

use std::{io, str, fmt};
use byteorder::*;
use error::*;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use walkdir::WalkDir;

struct Header {
    magic: &'static [u8],
    dir_offset: i32,
    dir_length: i32,
}

impl Header {
    fn read<R>(reader: &mut R) -> QResult<Header>
        where R: io::Read
    {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic);
        if &magic != b"PACK" {
            return Err(QError::BadMagicBytes);
        }
        let off = try!(reader.read_i32::<LittleEndian>());
        let len = try!(reader.read_i32::<LittleEndian>());

        Ok(Header {
            magic: b"PACK",
            dir_offset: off,
            dir_length: len,
        })
    }

    fn write<W>(&self, writer: &mut W) -> QResult<()>
        where W: io::Write
    {
        try!(writer.write_all(&self.magic));
        try!(writer.write_i32::<LittleEndian>(self.dir_offset));
        try!(writer.write_i32::<LittleEndian>(self.dir_length));
        
        Ok(())
    }
}

struct DirectoryEntry {
    name: [u8; 56],
    position: i32,
    length: i32,
}

impl DirectoryEntry {
    fn read<R>(reader: &mut R) -> QResult<DirectoryEntry>
        where R: io::Read
    {
        let mut name = [0; 56];
        try!(reader.read_exact(&mut name));

        let pos = try!(reader.read_i32::<LittleEndian>());
        let length = try!(reader.read_i32::<LittleEndian>());

        Ok(DirectoryEntry {
            name: name,
            position: pos,
            length: length,
        })
    }

    fn name_str(&self) -> &str {
        let name_bytes = &self.name;
        let nul = name_bytes.iter().position(|b| *b == 0).unwrap();
        let valid = &name_bytes[..nul];
        str::from_utf8(valid).unwrap()
    }
    
    fn write<W>(&self, writer: &mut W) -> QResult<()>
        where W: io::Write
    {
        try!(writer.write_all(&self.name));
        try!(writer.write_i32::<LittleEndian>(self.position));
        try!(writer.write_i32::<LittleEndian>(self.length));
        Ok(())
    }
}

impl fmt::Debug for DirectoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "DirectoryEntry {{ name: {}, position: {}, length: {} }}",
               self.name_str(),
               self.position,
               self.length)
    }
}

const DIR_ENTRY_SIZE: usize = 64;

pub struct PakFile {
    name: String,
    directory: Vec<DirectoryEntry>,
    reader: io::BufReader<File>
}

impl PakFile {
    pub fn read<P>(path: P) -> QResult<PakFile>
        where P: AsRef<Path>
    {
        let name = try!(path.as_ref()
                            .file_name()
                            .and_then(|s| s.to_str())
                            .ok_or(QError::BadFileName))
                       .into();
        let mut reader = io::BufReader::new(try!(File::open(path)));
        let header = try!(Header::read(&mut reader));
        try!(reader.seek(io::SeekFrom::Start(header.dir_offset as u64)));
        let file_count = header.dir_length as usize / DIR_ENTRY_SIZE;

        let mut entries = vec![];
        for _ in 0..file_count {
            let entry = try!(DirectoryEntry::read(&mut reader));
            entries.push(entry);
        }

        Ok(PakFile {
            name: name,
            directory: entries,
            reader: reader
        })
    }
    
    pub fn read_file(&mut self, name: &str) -> QResult<Vec<u8>> {
        let file = self.directory.iter().find(|f| f.name_str() == name);
        match file {
            Some(f) => {
                try!(self.reader.seek(io::SeekFrom::Start(f.position as u64)));
                let mut buf = vec![0; f.length as usize];
                let mut bytes_read = 0;
                while bytes_read < f.length {
                    bytes_read += try!(self.reader.read(&mut buf)) as i32;
                }
                Ok(buf)
                
            },
            None => Err(QError::FileNotFound)
        }
    }
}

pub fn create_pak<P: AsRef<Path>>(base_dir: P) -> QResult<PakFile> {
    let mut files = vec![];
    for entry in WalkDir::new(base_dir) {
        let entry = try!(entry);
        files.push(entry);
    }
    
    unimplemented!()
} 

#[cfg(test)]
mod tests {
    use super::PakFile;
    
    const PAK0: &'static str = "Id1/PAK0.PAK";
    
    #[test]
    fn read_pak() {
        let pak = PakFile::read(PAK0).unwrap();
        assert_eq!(pak.directory.len(), 339);
    }
}