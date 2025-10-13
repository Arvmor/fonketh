use crate::interface::{FPS, IDLE_DURATION, MAGIC_GROUND_SPEED, MAGIC_SPEED};
use crate::utils::Identifier;
use crate::world::World as WorldCore;
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
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
    pub fn run<I, B>(channel: Sender<KeyboardInput>, world: WorldCore<I, B>) -> Self
    where
        B: Clone + Eq + Hash + Send + Sync + 'static + Default,
        I: Send + Sync + 'static,
    {
        let sender = KeyEventSender(channel);
        let world = WorldState(world);

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(sender)
            .insert_resource(world)
            .insert_resource(SpawnedPlayers::default())
            .insert_resource(PlayerStates::default())
            .insert_resource(MiningRewards::default())
            .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
            .add_systems(Startup, setup::<B>)
            .add_systems(Update, capture_key_events)
            .add_systems(Update, check_shutdown_conditions::<I, B>) // Add this system
            .add_systems(Update, track_movement_events::<B>)
            .add_systems(Update, track_network_movements::<B>)
            .add_systems(Update, execute_animations::<B>)
            .add_systems(Update, spawn_new_players::<B>)
            .add_systems(Update, handle_idle_transitions::<B>)
            .add_systems(Update, update_ground_position::<B>)
            .add_systems(Update, track_mining_events::<B>)
            .add_systems(Update, update_status_bar)
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

/// Player state information bundled together for better cache locality
#[derive(Debug, Clone)]
struct PlayerStateInfo {
    state: CharacterState,
    last_movement_time: Instant,
    previous_position: (i64, i64),
    facing_right: bool,
}

impl Default for PlayerStateInfo {
    fn default() -> Self {
        Self {
            state: CharacterState::Idle,
            last_movement_time: Instant::now(),
            previous_position: (0, 0),
            facing_right: true, // Default to facing right
        }
    }
}

/// Resource to track player states and movement times within the interface
#[derive(Resource, Default)]
struct PlayerStates {
    players: HashMap<PeerId, PlayerStateInfo>,
}

/// Resource to track mining rewards counter
#[derive(Resource, Default)]
struct MiningRewards {
    count: u32,
}

/// Component to identify player entities
#[derive(Component)]
struct PlayerEntity {
    peer_id: PeerId,
}

/// Component to identify the ground entity
#[derive(Component)]
struct Ground;

/// Component to identify the status bar entity
#[derive(Component)]
struct StatusBar;

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
    let now = Instant::now();
    let local_player_id = world_state.0.identifier();

    for ev in evr_keys.read() {
        // Check if this is a movement key
        if matches!(
            ev.key_code,
            KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::ArrowUp | KeyCode::ArrowDown
        ) {
            // Get or create player state info
            let player_info = player_states.players.entry(local_player_id).or_default();

            // Update state to running
            player_info.state = CharacterState::Running;
            player_info.last_movement_time = now;

            // Update facing direction based on left/right movement
            match ev.key_code {
                KeyCode::ArrowRight => player_info.facing_right = true,
                KeyCode::ArrowLeft => player_info.facing_right = false,
                _ => {} // Don't change facing direction for up/down movement
            }
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
    let now = Instant::now();
    let all_players = world_state.0.get_all_players();
    let local_player_id = world_state.0.identifier();

    for (peer_id, character) in all_players {
        // Skip the local player as it's handled by track_movement_events
        if peer_id == local_player_id {
            continue;
        }

        let current_pos = (character.position.x, character.position.y);

        // Check if this player has moved
        let player_info = player_states.players.entry(peer_id).or_default();
        if player_info.previous_position != current_pos {
            // Player has moved, update state
            player_info.state = CharacterState::Running;
            player_info.last_movement_time = now;

            // Update facing direction based on horizontal movement
            if current_pos.0 > player_info.previous_position.0 {
                player_info.facing_right = true; // Moving right
            } else if current_pos.0 < player_info.previous_position.0 {
                player_info.facing_right = false; // Moving left
            }
        }

        // Always update the previous position
        player_info.previous_position = current_pos;
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

            // Get the player's state from the interface state tracking
            let player_info = player_states.players.get(&player_entity.peer_id);
            let player_state = player_info.map_or(CharacterState::Idle, |info| info.state.clone());
            let facing_right = player_info.map_or(true, |info| info.facing_right);

            // Apply sprite flipping based on facing direction

            if facing_right {
                transform.scale.x = 6.0; // Normal scale
            } else {
                transform.scale.x = -6.0; // Flipped scale (negative)
            }
            transform.scale.y = 6.0; // Keep Y scale normal

            let Some(atlas) = &mut sprite.texture_atlas else {
                continue;
            };

            // Handle animation based on character state
            match player_state {
                CharacterState::Idle => {
                    // Set to idle frame (frame 0) and stop animation
                    if atlas.index != 0 {
                        atlas.index = 0;
                    }
                    if !config.frame_timer.paused() {
                        config.frame_timer.pause();
                    }
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
    let all_players = world_state.0.get_all_players();

    // Check all players (both local and network) for idle transitions
    for peer_id in all_players.keys() {
        if let Some(player_info) = player_states.players.get_mut(peer_id)
            && now.duration_since(player_info.last_movement_time) > IDLE_DURATION
        {
            player_info.state = CharacterState::Idle;
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
                character.position.x as f32 * MAGIC_SPEED,
                character.position.y as f32 * MAGIC_SPEED,
                0.0,
            )),
            RightSprite,
            animation_config,
            PlayerEntity { peer_id },
        ));
    }
}

/// System to update ground position based on local player movement
fn update_ground_position<B>(
    world_state: Res<WorldState<PeerId, B>>,
    mut ground_query: Query<&mut Transform, With<Ground>>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let local_player_id = world_state.0.identifier();
    let all_players = world_state.0.get_all_players();

    // Get the local player's position
    if let Some(local_player) = all_players.get(&local_player_id) {
        let player_x = local_player.position.x as f32 * MAGIC_GROUND_SPEED;
        let player_y = local_player.position.y as f32 * MAGIC_GROUND_SPEED;

        // Update ground position to follow the player
        for mut ground_transform in ground_query.iter_mut() {
            ground_transform.translation.x = player_x;
            ground_transform.translation.y = player_y;
        }
    }
}

/// System to check for external shutdown conditions
fn check_shutdown_conditions<I, B>(
    mut writer: EventWriter<AppExit>,
    world_state: Res<WorldState<I, B>>,
) where
    I: Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    // Example: Check if the world's exit status is set
    if world_state.0.exit_status.is_exit() {
        info!("Shutting down Interface");
        writer.write(AppExit::Success);
    }
}

/// System to track mining events and update the counter
fn track_mining_events<B>(
    world_state: Res<WorldState<PeerId, B>>,
    mut mining_rewards: ResMut<MiningRewards>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    // Get the current mining rewards count from the world
    let world_count = world_state.0.get_mining_rewards_count();

    // Update our local counter if it's different
    if mining_rewards.count != world_count {
        mining_rewards.count = world_count;
    }
}

/// System to update the status bar display
fn update_status_bar(
    mining_rewards: Res<MiningRewards>,
    mut text_query: Query<&mut Text, With<StatusBar>>,
) {
    for mut text in text_query.iter_mut() {
        *text = Text::new(format!("Mining Rewards: {}", mining_rewards.count));
    }
}
