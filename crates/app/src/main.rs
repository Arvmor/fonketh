use game_core::prelude::*;
use game_core::world::{Character, World};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Initialize world
    let character = Character::new("John", 0);
    let world = World::new(character);
    world.initialize()?;

    Ok(())
}
