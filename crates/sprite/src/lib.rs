/// Character color constants
pub mod character;
/// Sprite image processing
pub mod image;
/// Integration tests
#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use character::*;
pub use image::{Color, SpriteImage};
