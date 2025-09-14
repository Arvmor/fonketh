use game_core::prelude::*;
use game_core::world::{Character, World};
use std::fs::File;
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::fmt()
        .with_ansi(false)
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(File::create("debug.log")?)
        .init();

    // Initialize world
    let character = Character::new("John", 0);
    let world = World::new(character);
    world.initialize()?;

    Ok(())
}
