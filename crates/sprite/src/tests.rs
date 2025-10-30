#[cfg(test)]
mod integration_tests {
    use crate::character::HAIR_COLOR;
    use crate::image::{Color, SpriteImage};
    use std::collections::HashMap;

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
