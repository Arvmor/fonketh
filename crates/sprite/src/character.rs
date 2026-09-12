use crate::Color;
use game_contract::prelude::keccak256;
use image::Rgba;

/// Hair Color
pub const HAIR_COLOR: Color = Color(Rgba([0x79, 0x3d, 0x4e, 0xff]));
/// Skin Color
pub const SKIN_COLOR: Color = Color(Rgba([0xfb, 0x95, 0x85, 0xff]));
/// Eyes Color
pub const EYES_COLOR: Color = Color(Rgba([0x85, 0xa3, 0xc7, 0xff]));
/// Clothing Color
pub const CLOTHING_COLOR: Color = Color(Rgba([0x01, 0x76, 0x87, 0xff]));

/// Character sprite sheet variants
///
/// Every variant shares the same 7x1 grid of 24x24 frames and uses
/// [`HAIR_COLOR`] for its hair or fur, so per-player recoloring applies to all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Character {
    Gabe,
    Female,
    Male,
    Alien,
    Ape,
    Cat,
    Wolf,
    ApeHumanized,
    CatHumanized,
    WolfHumanized,
}

impl Character {
    /// All available characters
    pub const ALL: [Character; 10] = [
        Character::Gabe,
        Character::Female,
        Character::Male,
        Character::Alien,
        Character::Ape,
        Character::Cat,
        Character::Wolf,
        Character::ApeHumanized,
        Character::CatHumanized,
        Character::WolfHumanized,
    ];

    /// Sprite sheet file name inside `assets/textures/characters`
    pub const fn file_name(self) -> &'static str {
        match self {
            Character::Gabe => "gabe-idle-run.png",
            Character::Female => "female-idle-run.png",
            Character::Male => "male-idle-run.png",
            Character::Alien => "alien-idle-run.png",
            Character::Ape => "ape-idle-run.png",
            Character::Cat => "cat-idle-run.png",
            Character::Wolf => "wolf-idle-run.png",
            Character::ApeHumanized => "ape-humanized-idle-run.png",
            Character::CatHumanized => "cat-humanized-idle-run.png",
            Character::WolfHumanized => "wolf-humanized-idle-run.png",
        }
    }

    /// Picks a character from an identifier
    ///
    /// Deterministic, so every peer renders the same character for a given player
    pub fn from_identifier(identifier: impl AsRef<[u8]>) -> Self {
        let hash = keccak256(identifier);
        Self::ALL[hash[0] as usize % Self::ALL.len()]
    }
}

/// Hat overlays for character sprite sheets
///
/// Hat sheets share the character grid and are transparent except for the hat,
/// so they can be composited straight onto a character sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hat {
    /// No hat
    Bare,
    MinerHelmet,
    Crown,
    TopHat,
    WizardHat,
    Cap,
}

impl Hat {
    /// All available hats, including going bare-headed
    pub const ALL: [Hat; 6] = [
        Hat::Bare,
        Hat::MinerHelmet,
        Hat::Crown,
        Hat::TopHat,
        Hat::WizardHat,
        Hat::Cap,
    ];

    /// Overlay file name inside `assets/textures/hats`, `None` when bare-headed
    pub const fn file_name(self) -> Option<&'static str> {
        match self {
            Hat::Bare => None,
            Hat::MinerHelmet => Some("miner_helmet.png"),
            Hat::Crown => Some("crown.png"),
            Hat::TopHat => Some("top_hat.png"),
            Hat::WizardHat => Some("wizard_hat.png"),
            Hat::Cap => Some("cap.png"),
        }
    }

    /// Picks a hat from an identifier
    ///
    /// Deterministic, so every peer renders the same hat for a given player
    pub fn from_identifier(identifier: impl AsRef<[u8]>) -> Self {
        let hash = keccak256(identifier);
        Self::ALL[hash[1] as usize % Self::ALL.len()]
    }
}
