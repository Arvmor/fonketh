use game_core::prelude::*;
use game_core::world::{B256, Character, Keypair, World};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            "app=debug,game_core=debug,game_contract=debug,game_network=debug,game_app=debug",
        )
        .init();

    // Read Arguments
    let load_key = std::env::args()
        .nth(1)
        .and_then(|f| std::fs::read_to_string(f).ok())
        .or(std::env::var("PRIVATE_KEY").ok())
        .expect("Provide Path to Private Key or set PRIVATE_KEY environment variable");

    // Load private key
    let private_key = load_key.parse::<B256>()?;
    let keypair = Keypair::ed25519_from_bytes(private_key)?;

    // Initialize world
    let character = Character::new(keypair.public().to_peer_id(), 0);
    let world = World::new(character);
    world.initialize(private_key.to_vec()).await?;

    Ok(())
}
