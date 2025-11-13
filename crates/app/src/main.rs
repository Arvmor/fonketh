use game_core::prelude::*;
use game_core::world::{B256, Character, Keypair, World};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            "game_core=debug,game_contract=debug,game_network=debug,game_app=debug,game_interface=debug,game_sprite=debug,game_api=debug",
        )
        .init();

    // Load private key
    let private_key = load_private_key()?;
    let keypair = Keypair::from_slice(&*private_key)?;

    // Initialize world
    let character = Character::new(keypair.address(), 0, (0, 0));
    let world = World::new(character);
    world.initialize(private_key.to_vec()).await?;

    Ok(())
}

/// Default path to private key file
const DEFAULT_KEY_PATH: &str = "./private.key";

/// Loads private key from environment variable, file argument, or default file.
fn load_private_key() -> Result<B256> {
    // Read Envs
    let env_key = std::env::var("PRIVATE_KEY");
    // Read Arg
    let arg_key = std::env::args().nth(1).map(std::fs::read_to_string);
    // Read Default Key
    let default_key = std::fs::read_to_string(DEFAULT_KEY_PATH);

    let key = match (env_key, arg_key, default_key) {
        (Ok(k), _, _) => k,
        (_, Some(Ok(k)), _) => k,
        (_, _, Ok(k)) => k,
        _ => {
            let new_key = B256::random().to_string();
            std::fs::write(DEFAULT_KEY_PATH, &new_key)?;
            new_key
        }
    };

    Ok(key.parse::<B256>()?)
}
