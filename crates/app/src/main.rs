use game_core::prelude::*;
use game_core::world::{Character, Keypair, World};
use std::fs::File;
use tracing_subscriber::filter::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let log_file = format!(
        "debug_{:?}.log",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?
    );
    tracing_subscriber::fmt::fmt()
        .with_ansi(false)
        .with_max_level(LevelFilter::DEBUG)
        .with_writer(File::create(log_file)?)
        .init();

    // Initialize world
    let arg = std::env::args().nth(1).unwrap().parse::<u8>()?;
    let keypair = Keypair::ed25519_from_bytes([arg; 32])?;
    let character = Character::new(keypair.public().to_peer_id(), 0);
    let world = World::new(character);
    world.initialize(keypair)?;

    Ok(())
}
