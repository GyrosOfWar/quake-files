use image::Rgb;
use error::*;
use std::io;
use byteorder::*;
use image::{DynamicImage, GenericImage};
use std::collections::HashSet;

pub type Color = Rgb<u8>;

#[derive(Debug)]
pub struct Palette {
    map: Vec<Color>,
}

impl Palette {
    pub fn read<R>(reader: &mut R) -> QResult<Palette>
        where R: io::Read
    {
        let mut buf = vec![Rgb { data: [0, 0, 0] }; 256];
        let mut data = vec![];
        try!(reader.read_to_end(&mut data));

        if data.len() > 256 * 3 {
            return Err(QError::InvalidPaletteSize);
        }

        for (i, c) in data.chunks(3).enumerate() {
            let (r, g, b) = (c[2], c[1], c[0]);
            buf[i] = Rgb { data: [r, g, b] };
        }

        Ok(Palette { map: buf })
    }

    pub fn write<W>(&self, writer: &mut W) -> QResult<()>
        where W: io::Write
    {
        for color in &self.map[..] {
            let (r, g, b) = (color[0], color[1], color[2]);
            try!(writer.write_u8(b));
            try!(writer.write_u8(g));
            try!(writer.write_u8(r));
        }

        Ok(())
    }

    pub fn from_image(image: &DynamicImage) -> QResult<Palette> {
        let mut colors = HashSet::new();
        for (_, _, pixel) in image.pixels() {
            colors.insert(pixel);
        }
        let mut bytes = vec![];
        for color in colors {
            bytes.push(color[2]);
            bytes.push(color[1]);
            bytes.push(color[0]);
        }
        let mut cursor = io::Cursor::new(bytes);
        Palette::read(&mut cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::Palette;
    use std::io;

    #[test]
    fn read_and_write() {
        let mut colors = vec![];

        for i in 0..256 {
            colors.push(i as u8);
            colors.push(i as u8);
            colors.push(i as u8);
        }

        let mut reader = io::Cursor::new(colors.clone());
        let palette = Palette::read(&mut reader).unwrap();

        for (i, color) in palette.map.iter().enumerate() {
            let i = i as u8;
            assert_eq!(color[0], i);
            assert_eq!(color[1], i);
            assert_eq!(color[2], i);
        }

        let mut out = vec![];
        palette.write(&mut out).unwrap();
        assert_eq!(out, colors);
    }
}
