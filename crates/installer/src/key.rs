//! Miner key setup
//!
//! The game reads `private.key` (a 32-byte hex string) from its home
//! directory and derives the reward wallet from it.

use crate::layout::{Layout, make_private};
use crate::ui::Ui;
use anyhow::{Result, bail};
use rand::RngCore;

/// Parses a pasted private key into its 64 hex characters, `0x` prefix optional
pub fn parse_hex_key(input: &str) -> Result<String> {
    let key: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let key = key
        .strip_prefix("0x")
        .or_else(|| key.strip_prefix("0X"))
        .unwrap_or(&key);
    if key.len() != 64 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("that is not a 64-character hex private key");
    }
    Ok(key.to_ascii_lowercase())
}

fn random_hex_key() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Step 5 of the wizard: keep, generate, import or defer the key
pub fn setup(layout: &Layout, ui: &Ui) -> Result<()> {
    let key_file = layout.key_file();
    if key_file.is_file() {
        Ui::ok(format!(
            "Keeping the existing miner key at {}",
            key_file.display()
        ));
        return Ok(());
    }

    Ui::say("Your rewards go to the wallet derived from this key.");
    let choice = ui.choose(
        "Miner key",
        &[
            "Generate a new key now (recommended)",
            "Import an existing private key",
            "Skip, the game creates one on first launch",
        ],
        0,
    )?;
    let key = match choice {
        1 => parse_hex_key(&ui.secret("Paste the 32-byte hex private key")?)?,
        2 => {
            Ui::note(format!(
                "The game will write {} on first launch",
                key_file.display()
            ));
            return Ok(());
        }
        _ => random_hex_key(),
    };

    std::fs::write(&key_file, format!("0x{key}"))?;
    make_private(&key_file)?;
    Ui::ok(format!("Miner key saved to {}", key_file.display()));
    Ui::warn(
        "Back this file up. Anyone with it controls your rewards; nobody can recover it for you.",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_with_and_without_prefix() {
        let raw = "AB".repeat(32);
        assert_eq!(parse_hex_key(&raw).unwrap(), "ab".repeat(32));
        assert_eq!(
            parse_hex_key(&format!("0x{raw}\n")).unwrap(),
            "ab".repeat(32)
        );
    }

    #[test]
    fn rejects_wrong_length_or_non_hex() {
        assert!(parse_hex_key("abc").is_err());
        assert!(parse_hex_key(&"zz".repeat(32)).is_err());
    }

    #[test]
    fn generated_keys_are_valid() {
        let key = random_hex_key();
        assert_eq!(parse_hex_key(&key).unwrap(), key);
    }
}
