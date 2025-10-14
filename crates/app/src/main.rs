use game_core::prelude::*;
use game_core::world::{Character, Keypair, World};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter("game_core=debug,game_contract=debug,game_network=debug,game_app=debug")
        .init();

    // Initialize world
    let arg = std::env::args().nth(1).unwrap().parse::<u8>()?;
    let keypair = Keypair::ed25519_from_bytes([arg; 32])?;
    let character = Character::new(keypair.public().to_peer_id(), 0);
    let world = World::new(character);
    world.initialize(&[arg; 32]).await?;

    Ok(())
}
