use game_contract::prelude::keccak256;
use image::{ImageReader, Rgba};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
};

/// A sprite image
pub struct SpriteImage {
    image: image::DynamicImage,
}

/// Hair Color
pub const HAIR_COLOR: Color = Color(Rgba([0x79, 0x3d, 0x4e, 0xff]));
pub const SKIN_COLOR: Color = Color(Rgba([0xfb, 0x95, 0x85, 0xff]));
pub const EYES_COLOR: Color = Color(Rgba([0x85, 0xa3, 0xc7, 0xff]));
pub const CLOTHING_COLOR: Color = Color(Rgba([0x01, 0x76, 0x87, 0xff]));

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Color(pub Rgba<u8>);

impl Color {
    /// Creates an RGBA from RGBA values
    pub fn create(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(Rgba([r, g, b, a]))
    }

    /// Creates an RGBA from a hex string
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

// Impl Ref for Color
impl AsRef<Rgba<u8>> for Color {
    fn as_ref(&self) -> &Rgba<u8> {
        &self.0
    }
}

impl SpriteImage {
    /// Builds a sprite image from an identifier
    pub fn from_identifier<I>(path: &Path, identifier: I) -> anyhow::Result<PathBuf>
    where
        I: AsRef<[u8]> + Display,
    {
        // Parse the path
        let ext = path.extension().and_then(OsStr::to_str);
        let name = path.file_prefix().and_then(OsStr::to_str);
        let output = match (name, ext) {
            (Some(name), Some(ext)) => path.with_file_name(format!("{name}-{identifier}.{ext}")),
            _ => return Err(anyhow::anyhow!("Invalid path")),
        };

        // Load the sprite image
        let color = Color::from_identifier(identifier);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_sprite_sheet() -> anyhow::Result<()> {
        SpriteImage::new("../../assets/textures/characters/gabe-idle-run.png")?;

        Ok(())
    }

    #[test]
    fn modify_color() -> anyhow::Result<()> {
        let path = "../../assets/textures/characters/gabe.png";
        let mut sprite_image = SpriteImage::new(path)?;

        // Create a map of colors to modify
        let map = HashMap::from([(HAIR_COLOR, Color::create(0x00, 0xc1, 0x9a, 0xff))]);

        // Modify the color of the sprite image
        sprite_image.modify_color(&map)?;
        sprite_image.save(path)?;

        Ok(())
    }
}
