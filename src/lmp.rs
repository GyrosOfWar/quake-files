//! Quake 1 style LMP image.

use crate::error::*;
use crate::palette::{Color, Palette};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};
use std::io;
use std::path::Path;

/// Quake 1 style LMP image. Does not store color values, only stores
/// indices into a color palette. (see `quake_files::palette::Palette`)
pub struct LmpImage {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl LmpImage {
    /// Opens the LMP image at the supplied `Read` instance.
    pub fn read<R>(reader: &mut R) -> QResult<LmpImage>
    where
        R: io::Read,
    {
        let width = reader.read_u32::<LittleEndian>()?;
        let height = reader.read_u32::<LittleEndian>()?;
        let mut bytes = vec![];
        reader.read_to_end(&mut bytes)?;

        if bytes.len() != (width * height) as usize {
            return Err(QError::InvalidLmp);
        }

        Ok(LmpImage {
            width: width,
            height: height,
            data: bytes,
        })
    }

    /// Writes the image to the supplied `Write` instance.
    pub fn write<W>(&self, writer: &mut W) -> QResult<()>
    where
        W: io::Write,
    {
        writer.write_u32::<LittleEndian>(self.width)?;
        writer.write_u32::<LittleEndian>(self.height)?;

        for byte in &self.data {
            writer.write_u8(*byte)?;
        }

        Ok(())
    }

    /// Creates a LMP image from a palette and some image. Returns an error if any of the image's
    /// colors are not in the palette.
    pub fn from_image(image: &DynamicImage, palette: &Palette) -> QResult<LmpImage> {
        let mut data = vec![];
        for (_, _, px) in image.pixels() {
            let palette_idx = palette
                .map()
                .iter()
                .position(|x| *x == px.to_rgb())
                .ok_or(QError::ColorNotInPalette)?;
            data.push(palette_idx as u8);
        }
        let width = image.width();
        let height = image.height();

        Ok(LmpImage {
            width: width,
            height: height,
            data: data,
        })
    }

    /// Saves the image to a file. See `image::ImageBuffer#save` for supported image formats.
    pub fn save_as<P>(&self, path: P, palette: Palette) -> QResult<()>
    where
        P: AsRef<Path>,
    {
        let colors: Vec<_> = self.data.iter().map(|px| palette.get(*px)).collect();
        let image = ImageBuffer::from_fn(self.width, self.height, |x, y| colors[self.index(x, y)]);
        image.save(path)?;
        Ok(())
    }

    #[inline]
    fn index(&self, y: u32, x: u32) -> usize {
        (x * self.width + y) as usize
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.data[self.index(x, y)]
    }

    /// Uses the given palette to translate an index in this image to a color.
    pub fn get_color(&self, x: u32, y: u32, palette: &Palette) -> Color {
        palette.get(self.get(x, y))
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
