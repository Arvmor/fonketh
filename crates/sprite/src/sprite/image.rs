use crate::sprite::character::HAIR_COLOR;
use game_contract::prelude::keccak256;
use image::{ImageReader, Rgba};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
};

/// Color - RGBA
///
/// Used to store a color in RGBA format
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Color(pub Rgba<u8>);

impl Color {
    /// Creates an RGBA from RGBA values
    pub fn create(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Rgba([r, g, b, a]))
    }

    /// Creates an RGBA from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::create(bytes[0], bytes[1], bytes[2], bytes[3])
    }

    /// Creates an RGBA from an identifier
    pub fn from_identifier(identifier: impl AsRef<[u8]>) -> Self {
        let hash = keccak256(identifier);
        let slice = hash.as_slice();

        // Create an RGBA from the hash
        Self::from_bytes(&[slice[28], slice[3], slice[12], 0xFF])
    }
}

impl AsRef<Rgba<u8>> for Color {
    fn as_ref(&self) -> &Rgba<u8> {
        &self.0
    }
}

/// A sprite image
pub struct SpriteImage {
    image: image::DynamicImage,
}

impl SpriteImage {
    /// Builds a sprite image from an identifier
    pub fn from_identifier<I>(path: &Path, id: I) -> anyhow::Result<PathBuf>
    where
        I: AsRef<[u8]> + Display,
    {
        // Parse the path
        let ext = path.extension().and_then(OsStr::to_str);
        let name = path.file_prefix().and_then(OsStr::to_str);
        let output = match (name, ext) {
            (Some(name), Some(ext)) => path.with_file_name(format!("mod_{name}-{id}.{ext}")),
            _ => return Err(anyhow::anyhow!("Invalid path")),
        };

        // Load the sprite image
        let color = Color::from_identifier(id);
        let mut sprite_image = Self::new(path)?;

        // Modify and Save
        sprite_image.modify_color(&HashMap::from([(HAIR_COLOR, color)]))?;
        sprite_image.save(&output)?;

        Ok(output)
    }

    /// Loads a sprite image from a file
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = ImageReader::open(&path)?.decode()?;
        Ok(Self { image })
    }

    /// Modify Color of the sprite image
    pub fn modify_color(&mut self, map: &HashMap<Color, Color>) -> anyhow::Result<()> {
        let colors = self
            .image
            .as_mut_rgba8()
            .ok_or(anyhow::anyhow!("Image is not a RGBA8 image"))?;

        for pixel in colors.pixels_mut() {
            if let Some(to) = map.get(&Color(*pixel)) {
                *pixel = *to.as_ref();
            }
        }

        Ok(())
    }

    /// Saves the sprite image to a file
    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.image.save(&path)?;
        Ok(())
    }
}
