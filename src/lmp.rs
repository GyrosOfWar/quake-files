use std::io;
// use std::io::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use error::*;
use palette::Palette;
use std::path::Path;
use image::ImageBuffer;

pub struct LmpImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl LmpImage {
    pub fn read<R>(reader: &mut R) -> QResult<LmpImage>
        where R: io::Read
    {
        let width = try!(reader.read_u32::<LittleEndian>());
        let height = try!(reader.read_u32::<LittleEndian>());
        let mut bytes = vec![];
        try!(reader.read_to_end(&mut bytes));

        if bytes.len() != (width * height) as usize {
            return Err(QError::InvalidLmp);
        }

        Ok(LmpImage {
            width: width,
            height: height,
            data: bytes,
        })
    }

    pub fn write<W>(&self, writer: &mut W) -> QResult<()>
        where W: io::Write
    {
        try!(writer.write_u32::<LittleEndian>(self.width));
        try!(writer.write_u32::<LittleEndian>(self.height));

        for byte in &self.data {
            try!(writer.write_u8(*byte));
        }

        Ok(())
    }
    
    pub fn save_as<P>(&self, path: P, palette: Palette) -> QResult<()>
        where P: AsRef<Path> 
    {
        let colors: Vec<_> = self.data.iter().map(|px| palette.get(*px)).collect();
        let image = ImageBuffer::from_fn(self.width, self.height, |x, y| {
            colors[(x * self.width + y) as usize]
        });
        try!(image.save(path));
        Ok(())
    }

    #[inline]
    fn index(&self, y: u32, x: u32) -> usize {
        (x * self.width + y) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.data[self.index(x, y)]
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.data
    }
}
