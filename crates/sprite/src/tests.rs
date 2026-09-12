#[cfg(test)]
mod integration_tests {
    use crate::character::{Character, HAIR_COLOR, Hat};
    use crate::image::{Color, SpriteImage};
    use std::collections::HashMap;
    use std::path::Path;

    const TEXTURES: &str = "../../assets/textures";

    #[test]
    fn load_sprite_sheet() -> anyhow::Result<()> {
        SpriteImage::new("../../assets/textures/characters/gabe-idle-run.png")?;
        Ok(())
    }

    #[test]
    fn modify_color() -> anyhow::Result<()> {
        let path = "../../assets/textures/characters/gabe-idle-run.png";
        let mut sprite_image = SpriteImage::new(path)?;

        // Create a map of colors to modify
        let map = HashMap::from([(HAIR_COLOR, Color::create(0x00, 0xc1, 0x9a, 0xff))]);

        // Modify the color of the sprite image
        sprite_image.modify_color(&map)?;
        let output = "../../assets/textures/characters/mod_gabe-idle-run-test.png";
        sprite_image.save(output)?;
        std::fs::remove_file(output)?;

        Ok(())
    }

    #[test]
    fn all_character_sheets_share_the_grid() -> anyhow::Result<()> {
        let reference = SpriteImage::new(
            Path::new(TEXTURES)
                .join("characters")
                .join(Character::Gabe.file_name()),
        )?
        .dimensions();

        for character in Character::ALL {
            let path = Path::new(TEXTURES)
                .join("characters")
                .join(character.file_name());
            let sheet = SpriteImage::new(&path)?;
            assert_eq!(sheet.dimensions(), reference, "{character:?} sheet size");
        }

        Ok(())
    }

    #[test]
    fn all_hat_sheets_overlay_on_characters() -> anyhow::Result<()> {
        let mut character = SpriteImage::new(
            Path::new(TEXTURES)
                .join("characters")
                .join(Character::Gabe.file_name()),
        )?;

        for hat in Hat::ALL {
            let Some(file_name) = hat.file_name() else {
                continue;
            };
            let hat_image = SpriteImage::new(Path::new(TEXTURES).join("hats").join(file_name))?;
            character.overlay(&hat_image)?;
        }

        Ok(())
    }

    #[test]
    fn appearance_is_deterministic() {
        let id = "12D3KooWExamplePeerId";
        assert_eq!(Character::from_identifier(id), Character::from_identifier(id));
        assert_eq!(Hat::from_identifier(id), Hat::from_identifier(id));
        assert_eq!(Color::from_identifier(id), Color::from_identifier(id));
    }

    #[test]
    fn appearance_covers_all_variants() {
        // Enough identifiers to hit every character and every hat
        let ids: Vec<String> = (0..2000).map(|i| format!("peer-{i}")).collect();
        for character in Character::ALL {
            assert!(
                ids.iter().any(|id| Character::from_identifier(id) == character),
                "{character:?} never selected"
            );
        }
        for hat in Hat::ALL {
            assert!(
                ids.iter().any(|id| Hat::from_identifier(id) == hat),
                "{hat:?} never selected"
            );
        }
    }

    #[test]
    fn build_sprite_from_identifier() -> anyhow::Result<()> {
        let output = SpriteImage::from_identifier(Path::new(TEXTURES), "test-player")?;
        assert!(output.exists());
        assert!(
            output
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("mod_") && n.ends_with("-test-player.png"))
        );
        std::fs::remove_file(output)?;
        Ok(())
    }
}
