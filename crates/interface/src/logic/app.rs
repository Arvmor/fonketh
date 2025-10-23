use crate::logic::{FPS, IDLE_DURATION, MAGIC_GROUND_SPEED, MAGIC_SPEED};
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use game_primitives::{Identifier, Player, Position, WorldState};
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
    pub fn run<W, P, I>(channel: Sender<KeyboardInput>, world: W) -> Self
    where
        W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
        P: Identifier<Id = I> + Player + Sync + Send + 'static,
        I: Sync + Send + 'static + Clone + Hash + Eq,
    {
        let sender = KeyEventSender(channel);
        let world = WorldStateResource(world);

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(sender)
            .insert_resource(world)
            .insert_resource(SpawnedPlayers::<P>::new())
            .insert_resource(PlayerStates::<P>::new())
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

/// Resource to send keyboard events to the core
#[derive(Resource)]
struct KeyEventSender(Sender<KeyboardInput>);

/// Resource to send world state to the interface
#[derive(Resource)]
struct WorldStateResource<W: WorldState + Send + Sync + 'static>(W);

/// Resource to track which players have been spawned in the UI
#[derive(Resource, Default)]
struct SpawnedPlayers<I: Identifier> {
    spawned: HashSet<I::Id>,
}

impl<I: Identifier> SpawnedPlayers<I> {
    fn new() -> Self {
        Self {
            spawned: Default::default(),
        }
    }
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
struct PlayerStates<I: Identifier> {
    players: HashMap<I::Id, PlayerStateInfo>,
}

impl<I: Identifier> PlayerStates<I> {
    fn new() -> Self {
        Self {
            players: Default::default(),
        }
    }
}

/// Resource to track mining rewards counter
#[derive(Resource, Default)]
struct MiningRewards {
    count: u32,
}

/// Component to identify player entities
#[derive(Component)]
struct PlayerEntity<I: Identifier> {
    peer_id: I::Id,
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

/// Tracks network player movements by comparing current positions with previous positions
fn track_network_movements<W, P, I>(
    world_state: Res<WorldStateResource<W>>,
    mut player_states: ResMut<PlayerStates<P>>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + 'static + Clone + Hash + Eq,
{
    let now = Instant::now();
    let all_players = world_state.0.get_all_players();

    for (peer_id, character) in all_players {
        let current_pos = (
            character.position().x() as i64,
            character.position().y() as i64,
        );

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
fn execute_animations<W, P, I>(
    time: Res<Time>,
    world_state: Res<WorldStateResource<W>>,
    player_states: Res<PlayerStates<P>>,
    mut player_query: Query<(
        &PlayerEntity<P>,
        &mut AnimationConfig,
        &mut Sprite,
        &mut Transform,
    )>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + 'static + Hash + Eq,
{
    let all_players = world_state.0.get_all_players();

    for (player_entity, mut config, mut sprite, mut transform) in player_query.iter_mut() {
        // Update position based on the character's position in the world state
        let Some(character) = all_players.get(&player_entity.peer_id) else {
            continue;
        };

        transform.translation.x = character.position().x() as f32 * MAGIC_SPEED;
        transform.translation.y = character.position().y() as f32 * MAGIC_SPEED;

        // Get the player's state from the interface state tracking
        let (state, facing_right) = player_states
            .players
            .get(&player_entity.peer_id)
            .map_or((CharacterState::Idle, true), |i| {
                (i.state.clone(), i.facing_right)
            });

        // Apply sprite flipping based on facing direction
        transform.scale.y = 6.0; // Keep Y scale normal
        if facing_right {
            transform.scale.x = 6.0; // Normal scale
        } else {
            transform.scale.x = -6.0; // Flipped scale (negative)
        }

        // Handle animation based on character state
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };
        match state {
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

/// System to handle transitions from running to idle state
/// This system automatically sets characters back to idle after they stop moving
fn handle_idle_transitions<W, P, I>(
    world_state: Res<WorldStateResource<W>>,
    mut player_states: ResMut<PlayerStates<P>>,
) where
    W: WorldState<Id = I> + Sync + Send + 'static,
    P: Identifier<Id = I> + Sync + Send + 'static,
    I: Sync + Send + 'static + Clone + Hash + Eq,
{
    let all_players = world_state.0.get_all_players();

    // Check all players (both local and network) for idle transitions
    for id in all_players.keys() {
        if let Some(state) = player_states.players.get_mut(id)
            && state.last_movement_time.elapsed() > IDLE_DURATION
        {
            state.state = CharacterState::Idle;
        }
    }
}

#[derive(Component)]
struct RightSprite;

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

/// System to update ground position based on local player movement
fn update_ground_position<W, P, I>(
    world_state: Res<WorldStateResource<W>>,
    mut ground_query: Query<&mut Transform, With<Ground>>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + 'static + Clone + Hash + Eq,
{
    let local_player_id = world_state.0.identifier();
    let all_players = world_state.0.get_all_players();

    // Get the local player's position
    if let Some(local_player) = all_players.get(&local_player_id) {
        let player_x = local_player.position().x() as f32 * MAGIC_GROUND_SPEED;
        let player_y = local_player.position().y() as f32 * MAGIC_GROUND_SPEED;

        // Update ground position to follow the player
        for mut ground_transform in ground_query.iter_mut() {
            ground_transform.translation.x = player_x;
            ground_transform.translation.y = player_y;
        }
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

/// System to track mining events and update the counter
fn track_mining_events<W: WorldState + Sync + Send + 'static>(
    world_state: Res<WorldStateResource<W>>,
    mut mining_rewards: ResMut<MiningRewards>,
) {
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
