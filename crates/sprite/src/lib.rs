use image::{ImageReader, Rgba};
use std::{collections::HashMap, path::Path};

/// A sprite image
pub struct SpriteImage {
    image: image::DynamicImage,
}

/// Hair Color
pub const HAIR_COLOR: Rgba<u8> = Rgba([0x79, 0x3d, 0x4e, 0xff]);
pub const SKIN_COLOR: Rgba<u8> = Rgba([0xfb, 0x95, 0x85, 0xff]);
pub const EYES_COLOR: Rgba<u8> = Rgba([0x85, 0xa3, 0xc7, 0xff]);
pub const CLOTHING_COLOR: Rgba<u8> = Rgba([0x01, 0x76, 0x87, 0xff]);

#[derive(Debug)]
pub struct Color(pub Rgba<u8>);

impl Color {
    pub fn create(r: u8, g: u8, b: u8, a: u8) -> Rgba<u8> {
        Rgba([r, g, b, a])
    }

    /// Creates an RGBA from a hex string
    pub fn from_bytes(bytes: &[u8]) -> Rgba<u8> {
        Self::create(bytes[0], bytes[1], bytes[2], bytes[3])
    }
}

impl SpriteImage {
    /// Loads a sprite image from a file
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = ImageReader::open(&path)?.decode()?;
        Ok(Self { image })
    }

    /// Modify Color of the sprite image
    pub fn modify_color(&mut self, map: &HashMap<Rgba<u8>, Rgba<u8>>) -> anyhow::Result<()> {
        let colors = self
            .image
            .as_mut_rgba8()
            .ok_or(anyhow::anyhow!("Image is not a RGBA8 image"))?;

        for pixel in colors.pixels_mut() {
            if let Some(to) = map.get(pixel) {
                *pixel = *to;
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
        let map = HashMap::from([(HAIR_COLOR, Rgba([0x00, 0xc1, 0x9a, 0xff]))]);

        // Modify the color of the sprite image
        sprite_image.modify_color(&map)?;
        sprite_image.save(path)?;

        Ok(())
    }
}
