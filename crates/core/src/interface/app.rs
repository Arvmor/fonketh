use crate::utils::Identifier;
use crate::world::World as WorldCore;
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::winit::{WakeUp, WinitPlugin};
use game_network::prelude::PeerId;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Default)]
enum CharacterState {
    #[default]
    Idle,
    Running,
}

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
    pub fn run<B>(channel: Sender<KeyboardInput>, world: WorldCore<PeerId, B>) -> Self
    where
        B: Clone + Eq + Hash + Send + Sync + 'static + Default,
    {
        let sender = KeyEventSender(channel);
        let world = WorldState(world);

        // Run on any thread
        let mut winit = WinitPlugin::<WakeUp>::default();
        winit.run_on_any_thread = true;

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(sender)
            .insert_resource(world)
            .insert_resource(SpawnedPlayers::default())
            .insert_resource(PlayerStates::default())
            .add_plugins(
                DefaultPlugins
                    .set(ImagePlugin::default_nearest())
                    .set(winit),
            ) // prevents blurry sprites
            .add_systems(Startup, setup::<B>)
            .add_systems(Update, capture_key_events)
            .add_systems(Update, track_movement_events::<B>)
            .add_systems(Update, track_network_movements::<B>)
            .add_systems(Update, execute_animations::<B>)
            .add_systems(Update, spawn_new_players::<B>)
            .add_systems(Update, handle_idle_transitions::<B>)
            .run();

        Self { app }
    }
}

/// Resource to send keyboard events to the core
#[derive(Resource)]
struct KeyEventSender(Sender<KeyboardInput>);

/// Resource to send world state to the interface
#[derive(Resource)]
struct WorldState<I, B>(WorldCore<I, B>);

/// Resource to track which players have been spawned in the UI
#[derive(Resource, Default)]
struct SpawnedPlayers {
    spawned: HashSet<PeerId>,
}

/// Resource to track player states and movement times within the interface
#[derive(Resource, Default)]
struct PlayerStates {
    states: HashMap<PeerId, CharacterState>,
    last_movement_times: HashMap<PeerId, Instant>,
    previous_positions: HashMap<PeerId, (i64, i64)>,
}

/// Component to identify player entities
#[derive(Component)]
struct PlayerEntity {
    peer_id: PeerId,
}

/// Captures keyboard events and sends them to the core channel
fn capture_key_events(mut evr_keys: EventReader<KeyboardInput>, sender: Res<KeyEventSender>) {
    for ev in evr_keys.read() {
        info!("Keyboard event: {ev:?}");

        // Send over channel to core
        if let Err(e) = sender.0.send(ev.clone()) {
            error!("Error sending keyboard event: {e:?}");
        }
    }
}

/// Tracks movement events and updates player states within the interface
fn track_movement_events<B>(
    mut evr_keys: EventReader<KeyboardInput>,
    world_state: Res<WorldState<PeerId, B>>,
    mut player_states: ResMut<PlayerStates>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let local_player_id = world_state.0.identifier();

    for ev in evr_keys.read() {
        // Check if this is a movement key
        if matches!(
            ev.key_code,
            KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
        ) {
            // Update the local player's state to running
            player_states
                .states
                .insert(local_player_id, CharacterState::Running);
            player_states
                .last_movement_times
                .insert(local_player_id, Instant::now());
        }
    }
}

/// Tracks network player movements by comparing current positions with previous positions
fn track_network_movements<B>(
    world_state: Res<WorldState<PeerId, B>>,
    mut player_states: ResMut<PlayerStates>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let all_players = world_state.0.get_all_players();
    let local_player_id = world_state.0.identifier();

    for (peer_id, character) in all_players {
        // Skip the local player as it's handled by track_movement_events
        if peer_id == local_player_id {
            continue;
        }

        let current_pos = (character.position.x, character.position.y);

        // Check if this player has moved
        if let Some(previous_pos) = player_states.previous_positions.get(&peer_id)
            && *previous_pos != current_pos
        {
            // Player has moved, set to running state
            player_states
                .states
                .insert(peer_id, CharacterState::Running);
            player_states
                .last_movement_times
                .insert(peer_id, Instant::now());
        }

        // Update the previous position
        player_states
            .previous_positions
            .insert(peer_id, current_pos);
    }
}

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

/// Combined system to update player positions and execute animations
/// This system handles both position updates and animation frame progression for all players
fn execute_animations<B>(
    time: Res<Time>,
    world_state: Res<WorldState<PeerId, B>>,
    player_states: Res<PlayerStates>,
    mut player_query: Query<(
        &PlayerEntity,
        &mut AnimationConfig,
        &mut Sprite,
        &mut Transform,
    )>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let all_players = world_state.0.get_all_players();

    for (player_entity, mut config, mut sprite, mut transform) in player_query.iter_mut() {
        // Update position based on the character's position in the world state
        if let Some(character) = all_players.get(&player_entity.peer_id) {
            transform.translation.x = character.position.x as f32 * 24.0;
            transform.translation.y = character.position.y as f32 * 24.0;

            let Some(atlas) = &mut sprite.texture_atlas else {
                continue;
            };

            // Get the player's state from the interface state tracking
            let player_state = player_states
                .states
                .get(&player_entity.peer_id)
                .cloned()
                .unwrap_or(CharacterState::Idle);

            // Handle animation based on character state
            match player_state {
                CharacterState::Idle => {
                    // Set to idle frame (frame 0) and stop animation
                    atlas.index = 0;
                    config.frame_timer.pause();
                }
                CharacterState::Running => {
                    // Resume animation timer if paused
                    if config.frame_timer.paused() {
                        config.frame_timer.unpause();
                    }

                    // Handle animation frame progression
                    config.frame_timer.tick(time.delta());

                    // If it has been displayed for the user-defined amount of time (fps)...
                    if config.frame_timer.just_finished() {
                        // If last frame, then we move back to the first frame and stop.
                        if atlas.index == config.last_sprite_index {
                            atlas.index = config.first_sprite_index;
                        } else {
                            // Move to next frame
                            atlas.index += 1;
                        }

                        // Reset the frame timer to start counting all over again
                        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                    }
                }
            }
        }
    }
}

/// System to handle transitions from running to idle state
/// This system automatically sets characters back to idle after they stop moving
fn handle_idle_transitions<B>(
    world_state: Res<WorldState<PeerId, B>>,
    mut player_states: ResMut<PlayerStates>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let now = Instant::now();
    let idle_duration = Duration::from_millis(200); // 200ms delay before going idle
    let all_players = world_state.0.get_all_players();

    // Check all players (both local and network) for idle transitions
    for peer_id in all_players.keys() {
        if let Some(last_movement) = player_states.last_movement_times.get(peer_id)
            && now.duration_since(*last_movement) > idle_duration
        {
            player_states.states.insert(*peer_id, CharacterState::Idle);
        }
    }
}

#[derive(Component)]
struct RightSprite;

fn setup<B>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut spawned_players: ResMut<SpawnedPlayers>,
    world_state: Res<WorldState<PeerId, B>>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    commands.spawn(Camera2d);

    // Load the sprite sheet using the `AssetServer`
    let image = asset_server.load("textures/characters/gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let layout = texture_atlas_layouts.add(texture_atlas_layout);

    // The first (left-hand) sprite runs at 10 FPS
    let animation_config_1 = AnimationConfig::new(1, 6, 20);
    let index = animation_config_1.first_sprite_index;
    let texture_atlas = Some(TextureAtlas { layout, index });

    // Create the first (left-hand) sprite
    let peer_id = world_state.0.identifier();
    spawned_players.spawned.insert(peer_id);
    commands.spawn((
        Sprite {
            image,
            texture_atlas,
            ..default()
        },
        Transform::from_scale(Vec3::splat(6.0)),
        RightSprite,
        animation_config_1,
        PlayerEntity { peer_id },
    ));
}

/// System to spawn new player characters in the UI
fn spawn_new_players<B>(
    mut commands: Commands,
    world_state: Res<WorldState<PeerId, B>>,
    mut spawned_players: ResMut<SpawnedPlayers>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let all_players = world_state.0.get_all_players();

    for (peer_id, character) in all_players {
        // If the player has already been spawned, skip
        if !spawned_players.spawned.insert(peer_id) {
            continue;
        }

        // Load the sprite sheet using the `AssetServer`
        let image = asset_server.load("textures/characters/gabe-idle-run.png");
        let texture_atlas_layout =
            TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
        let layout = texture_atlas_layouts.add(texture_atlas_layout);

        // Create animation config for this player
        let animation_config = AnimationConfig::new(1, 6, 20);
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
                character.position.x as f32 * 24.0, // Convert to pixel coordinates
                character.position.y as f32 * 24.0,
                0.0,
            )),
            RightSprite,
            animation_config,
            PlayerEntity { peer_id },
        ));
    }
}
