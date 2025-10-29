use image::{ImageReader, Rgba};
use std::{collections::HashMap, path::Path};

/// A sprite image
pub struct SpriteImage<P> {
    path: P,
    image: image::DynamicImage,
}

/// Hair Color
pub const HAIR_COLOR: Rgba<u8> = Rgba([0x79, 0x3d, 0x4e, 0xff]);
pub const SKIN_COLOR: Rgba<u8> = Rgba([0xfb, 0x95, 0x85, 0xff]);
pub const EYES_COLOR: Rgba<u8> = Rgba([0x85, 0xa3, 0xc7, 0xff]);
pub const CLOTHING_COLOR: Rgba<u8> = Rgba([0x01, 0x76, 0x87, 0xff]);

impl<P: AsRef<Path>> SpriteImage<P> {
    /// Loads a sprite image from a file
    pub fn new(path: P) -> anyhow::Result<Self> {
        let image = ImageReader::open(&path)?.decode()?;
        Ok(Self { path, image })
    }

    /// Modify Color of the sprite image
    pub fn modify_color(&mut self, map: HashMap<Rgba<u8>, Rgba<u8>>) -> anyhow::Result<()> {
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
    pub fn save(&self) -> anyhow::Result<()> {
        self.image.save(&self.path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_sprite_sheet() -> anyhow::Result<()> {
        SpriteImage::new("../app/assets/textures/characters/gabe-idle-run.png")?;

        Ok(())
    }

    #[test]
    fn modify_color() -> anyhow::Result<()> {
        let mut sprite_image = SpriteImage::new("../app/assets/textures/characters/gabe.png")?;

        // Create a map of colors to modify
        let map = HashMap::from([(HAIR_COLOR, Rgba([0x00, 0xc1, 0x9a, 0xff]))]);

        // Modify the color of the sprite image
        sprite_image.modify_color(map)?;
        sprite_image.save()?;

        Ok(())
    }
}
