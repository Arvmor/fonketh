use crate::minings::{track_mining_events, update_status_bar};
use crate::movements::{
    capture_key_events, execute_animations, follow_main_player_with_camera,
    handle_idle_transitions, track_network_movements,
};
use crate::prelude::*;
use bevy::prelude::*;
use game_primitives::events::GameEvent;
use game_primitives::{Identifier, Player, Position, WorldState};
use std::fmt::Display;
use std::hash::Hash;
use std::path::Path;
use std::sync::mpsc::Sender;

/// Interface for the game
///
/// responsible for managing Bevy app and Keyboard events
pub struct Interface {
    pub app: AppExit,
}

impl Interface {
    /// Runs the Bevy app
    ///
    /// Creates a new Bevy app and runs it
    pub fn run<W, P, I, F, Po>(channel: Sender<GameEvent<F, Po>>, world: W) -> Self
    where
        F: Send + Sync + 'static,
        Po: Position<Unit = i32> + Send + Sync + 'static,
        W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
        P: Identifier<Id = I> + Player + Sync + Send + 'static,
        I: Hash + Eq + Clone + Sync + Send + Display + 'static,
    {
        // Config plugins
        let image_plugin = ImagePlugin::default_nearest();
        let asset_plugin = AssetPlugin {
            file_path: "./../..".to_string(),
            ..Default::default()
        };

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(KeyEventSender(channel))
            .insert_resource(WorldStateResource(world))
            .insert_resource(SpawnedPlayers::<P>::default())
            .insert_resource(PlayerStates::<P>::default())
            .insert_resource(MiningRewards::default())
            .insert_resource(ChatInputText::default())
            // prevents blurry sprites
            .add_plugins(DefaultPlugins.set(image_plugin).set(asset_plugin))
            // Startup systems
            .add_systems(Startup, setup)
            .add_systems(Startup, setup_hud)
            // Update systems
            .add_systems(Update, capture_key_events::<F, Po>)
            .add_systems(Update, check_shutdown_conditions::<W>)
            .add_systems(Update, track_network_movements::<W, P, I>)
            .add_systems(Update, execute_animations::<W, P, I>)
            .add_systems(Update, spawn_new_players::<W, P, I>)
            .add_systems(Update, handle_idle_transitions::<W, P, I>)
            .add_systems(Update, follow_main_player_with_camera)
            .add_systems(Update, track_mining_events::<W>)
            .add_systems(Update, update_status_bar)
            .add_systems(Update, update_player_count::<W>)
            .add_systems(Update, handle_chat_input::<F, Po>)
            .add_systems(Update, display_chat_messages::<W>)
            .run();

        Self { app }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Spawn the grass background
    let image = asset_server.load("./assets/textures/background/full.png");
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_translation(Vec3::new(0., 0., -1.)).with_scale(Vec3::splat(1.5)),
        Ground,
    ));
}

/// System to spawn new player characters in the UI
fn spawn_new_players<W, P, I>(
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    mut spawned_players: ResMut<SpawnedPlayers<P>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + Display + 'static,
{
    // Plain Character Sprite Path
    let path = Path::new("./assets/textures/characters/gabe-idle-run.png");
    let local_player_id = world_state.0.identifier();

    for (peer_id, character) in world_state.0.get_all_players() {
        // If the player has already been spawned, skip
        if !spawned_players.spawned.insert(peer_id.clone()) {
            continue;
        }

        // Modify the sprite image based on the player's color
        #[cfg(feature = "custom_sprites")]
        let path = game_sprite::SpriteImage::from_identifier(path, peer_id.to_string())
            .unwrap_or_else(|e| {
                error!("Failed to modify sprite image: {e}");
                path.to_path_buf()
            });

        // Load the sprite sheet using the `AssetServer`
        let image = asset_server.load(path);
        let texture_atlas_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
        let layout = texture_atlas_layouts.add(texture_atlas_layout);

        // Create animation config for this player
        let animation_config = AnimationConfig::new(1, 6, FPS);
        let index = animation_config.first_sprite_index;
        let texture_atlas = Some(TextureAtlas { layout, index });

        // Check if this is the main/local player
        let is_main_player = peer_id == local_player_id;

        // Spawn the player character
        let mut entity_commands = commands.spawn((
            Sprite {
                image,
                texture_atlas,
                ..default()
            },
            Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(
                character.position().x() as f32 * MAGIC_SPEED,
                character.position().y() as f32 * MAGIC_SPEED,
                0.0,
            )),
            RightSprite,
            animation_config,
            PlayerEntity::<P> { peer_id },
        ));

        // Add MainPlayer component if this is the local player
        if is_main_player {
            entity_commands.insert(MainPlayer);
        }
    }
}

/// System to check for external shutdown conditions
fn check_shutdown_conditions<W: WorldState + Sync + Send + 'static>(
    mut writer: MessageWriter<AppExit>,
    world_state: Res<WorldStateResource<W>>,
) {
    // Example: Check if the world's exit status is set
    if world_state.0.exit_status().is_exit() {
        info!("Shutting down Interface");
        writer.write(AppExit::Success);
    }
}
