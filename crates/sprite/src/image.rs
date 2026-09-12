use crate::character::{Character, HAIR_COLOR, Hat};
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
    /// Builds a player's sprite sheet from an identifier
    ///
    /// `textures` is the textures directory holding `characters/` and `hats/`.
    /// The character, hat and hair color are all derived from the identifier hash,
    /// and the composed sheet is saved next to the character sheets as
    /// `mod_{character}-{id}.png`.
    pub fn from_identifier<I>(textures: &Path, id: I) -> anyhow::Result<PathBuf>
    where
        I: AsRef<[u8]> + Display,
    {
        let character = Character::from_identifier(&id);
        let hat = Hat::from_identifier(&id);
        let color = Color::from_identifier(&id);

        // Resolve the paths
        let characters = textures.join("characters");
        let base = characters.join(character.file_name());
        let output = match (base.file_stem().and_then(OsStr::to_str), base.extension()) {
            (Some(name), Some(ext)) => {
                characters.join(format!("mod_{name}-{id}.{}", ext.to_string_lossy()))
            }
            _ => return Err(anyhow::anyhow!("Invalid path")),
        };

        // Load the character, recolor the hair and put the hat on
        let mut sprite_image = Self::new(&base)?;
        sprite_image.modify_color(&HashMap::from([(HAIR_COLOR, color)]))?;
        if let Some(file_name) = hat.file_name() {
            let hat_image = Self::new(textures.join("hats").join(file_name))?;
            sprite_image.overlay(&hat_image)?;
        }

        sprite_image.save(&output)?;
        Ok(output)
    }

    /// Loads a sprite image from a file
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let image = ImageReader::open(&path)?.decode()?;
        Ok(Self { image })
    }

    /// Image dimensions as (width, height)
    pub fn dimensions(&self) -> (u32, u32) {
        (self.image.width(), self.image.height())
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

    /// Draws a layer of the same size on top of the sprite image
    ///
    /// Every non-transparent pixel of the layer replaces the pixel underneath.
    pub fn overlay(&mut self, layer: &SpriteImage) -> anyhow::Result<()> {
        if layer.dimensions() != self.dimensions() {
            return Err(anyhow::anyhow!(
                "Layer dimensions {:?} do not match sprite dimensions {:?}",
                layer.dimensions(),
                self.dimensions()
            ));
        }

        let base = self
            .image
            .as_mut_rgba8()
            .ok_or(anyhow::anyhow!("Image is not a RGBA8 image"))?;
        let layer = layer.image.to_rgba8();

        for (pixel, top) in base.pixels_mut().zip(layer.pixels()) {
            if top.0[3] != 0 {
                *pixel = *top;
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
