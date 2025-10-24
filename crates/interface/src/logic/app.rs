use crate::minings::{track_mining_events, update_status_bar};
use crate::movements::{
    capture_key_events, execute_animations, handle_idle_transitions, track_network_movements,
    update_ground_position,
};
use crate::prelude::*;
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use game_primitives::{Identifier, Player, Position, WorldState};
use std::hash::Hash;
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
    pub fn run<W, P, I>(channel: Sender<KeyboardInput>, world: W) -> Self
    where
        W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
        P: Identifier<Id = I> + Player + Default + Sync + Send + 'static,
        I: Hash + Eq + Default + Clone + Sync + Send + 'static,
    {
        let sender = KeyEventSender(channel);
        let world = WorldStateResource(world);

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(sender)
            .insert_resource(world)
            .insert_resource(SpawnedPlayers::<P>::default())
            .insert_resource(PlayerStates::<P>::default())
            .insert_resource(MiningRewards::default())
            .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
            .add_systems(Startup, setup::<W, P, I>)
            .add_systems(Update, capture_key_events)
            .add_systems(Update, check_shutdown_conditions::<W>) // Add this system
            .add_systems(Update, track_network_movements::<W, P, I>)
            .add_systems(Update, execute_animations::<W, P, I>)
            .add_systems(Update, spawn_new_players::<W, P, I>)
            .add_systems(Update, handle_idle_transitions::<W, P, I>)
            .add_systems(Update, update_ground_position::<W, P, I>)
            .add_systems(Update, track_mining_events::<W>)
            .add_systems(Update, update_status_bar)
            .run();

        Self { app }
    }
}

fn setup<W, P, I>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut spawned_players: ResMut<SpawnedPlayers<P>>,
    world_state: Res<WorldStateResource<W>>,
) where
    W: WorldState<Id = I> + Sync + Send + 'static,
    P: Identifier<Id = I> + Sync + Send + 'static,
    I: Sync + Send + 'static + Clone + Hash + Eq,
{
    commands.spawn(Camera2d);

    // Spawn the status bar in the top left
    commands.spawn((
        Text::new("Mining Rewards: 0"),
        Transform::from_translation(Vec3::new(-400.0, 300.0, 10.0)),
        StatusBar,
    ));

    // Spawn the grass background
    let image = asset_server.load("textures/background/full.png");
    commands.spawn((
        Sprite { image, ..default() },
        Transform::from_translation(Vec3::new(0., 0., -1.)).with_scale(Vec3::splat(1.5)),
        Ground,
    ));

    // Load the sprite sheet using the `AssetServer`
    let image = asset_server.load("textures/characters/gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let layout = texture_atlas_layouts.add(texture_atlas_layout);

    // The first (left-hand) sprite runs at 10 FPS
    let animation_config_1 = AnimationConfig::new(1, 6, FPS);
    let index = animation_config_1.first_sprite_index;
    let texture_atlas = Some(TextureAtlas { layout, index });

    // Create the first (left-hand) sprite
    let peer_id = world_state.0.identifier();
    spawned_players.spawned.insert(peer_id.clone());
    commands.spawn((
        Sprite {
            image,
            texture_atlas,
            ..default()
        },
        Transform::from_scale(Vec3::splat(6.0)),
        RightSprite,
        animation_config_1,
        PlayerEntity::<P> { peer_id },
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
    I: Sync + Send + 'static + Clone + Hash + Eq,
{
    let all_players = world_state.0.get_all_players();

    for (peer_id, character) in all_players {
        // If the player has already been spawned, skip
        if !spawned_players.spawned.insert(peer_id.clone()) {
            continue;
        }

        // Load the sprite sheet using the `AssetServer`
        let image = asset_server.load("textures/characters/gabe-idle-run.png");
        let texture_atlas_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
        let layout = texture_atlas_layouts.add(texture_atlas_layout);

        // Create animation config for this player
        let animation_config = AnimationConfig::new(1, 6, FPS);
        let index = animation_config.first_sprite_index;
        let texture_atlas = Some(TextureAtlas { layout, index });

        // Spawn the player character
        commands.spawn((
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
    }
}

/// System to check for external shutdown conditions
fn check_shutdown_conditions<W: WorldState + Sync + Send + 'static>(
    mut writer: EventWriter<AppExit>,
    world_state: Res<WorldStateResource<W>>,
) {
    // Example: Check if the world's exit status is set
    if world_state.0.exit_status().is_exit() {
        info!("Shutting down Interface");
        writer.write(AppExit::Success);
    }
}
