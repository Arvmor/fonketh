use crate::utils::Identifier;
use crate::world::World as WorldCore;
use bevy::input::common_conditions::input_just_pressed;
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::winit::{WakeUp, WinitPlugin};
use game_network::prelude::PeerId;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::mpsc::Sender;
use std::time::Duration;

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
            .add_plugins(
                DefaultPlugins
                    .set(ImagePlugin::default_nearest())
                    .set(winit),
            ) // prevents blurry sprites
            .add_systems(Startup, setup::<B>)
            .add_systems(Update, capture_key_events)
            .add_systems(Update, execute_animations)
            .add_systems(Update, spawn_new_players::<B>)
            .add_systems(Update, update_player_positions::<B>)
            .add_systems(
                Update,
                trigger_animation::<RightSprite>.run_if(input_just_pressed(KeyCode::ArrowRight)),
            )
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

// This system runs when the user clicks the left arrow key or right arrow key
fn trigger_animation<S: Component>(mut animation: Single<&mut AnimationConfig, With<S>>) {
    // We create a new timer when the animation is triggered
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
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

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut Sprite, &mut Transform)>,
) {
    for (mut config, mut sprite, mut transform) in &mut query {
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if !config.frame_timer.just_finished() {
            continue;
        }

        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        // If last frame, then we move back to the first frame and stop.
        if atlas.index == config.last_sprite_index {
            atlas.index = config.first_sprite_index;
            continue;
        }

        // Reset the frame timer to start counting all over again
        atlas.index += 1;
        transform.translation.x += 1500. * time.delta_secs();
        config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
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
    spawned_players.spawned.insert(world_state.0.identifier());
    commands.spawn((
        Sprite {
            image,
            texture_atlas,
            ..default()
        },
        Transform::from_scale(Vec3::splat(6.0)),
        RightSprite,
        animation_config_1,
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

/// System to update existing player positions in the UI
fn update_player_positions<B>(
    world_state: Res<WorldState<PeerId, B>>,
    mut player_query: Query<(&PlayerEntity, &mut Transform)>,
) where
    B: Clone + Eq + Hash + Send + Sync + 'static + Default,
{
    let all_players = world_state.0.get_all_players();

    for (player_entity, mut transform) in player_query.iter_mut() {
        if let Some(character) = all_players.get(&player_entity.peer_id) {
            // Update the transform position based on the character's position
            transform.translation.x = character.position.x as f32 * 24.0;
            transform.translation.y = character.position.y as f32 * 24.0;
        }
    }
}
