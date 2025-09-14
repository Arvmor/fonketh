use game_core::prelude::*;
use game_core::world::{Character, World};

fn main() -> Result<()> {
    let character = Character::new("John", 0);
    let world = World::new("1", vec![character]);

    Ok(())
}
