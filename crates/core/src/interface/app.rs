use bevy::input::common_conditions::input_just_pressed;
pub use bevy::input::keyboard::{KeyCode, KeyboardInput};
use bevy::prelude::*;
use bevy::winit::{WakeUp, WinitPlugin};
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
    pub fn run(channel: Sender<KeyboardInput>) -> Self {
        let sender = KeyEventSender(channel);

        // Run on any thread
        let mut winit = WinitPlugin::<WakeUp>::default();
        winit.run_on_any_thread = true;

        let app = App::new()
            // Channel to pass Events to core
            .insert_resource(sender)
            .add_plugins(
                DefaultPlugins
                    .set(ImagePlugin::default_nearest())
                    .set(winit),
            ) // prevents blurry sprites
            .add_systems(Startup, setup)
            .add_systems(Update, capture_key_events)
            .add_systems(Update, execute_animations)
            .add_systems(
                Update,
                trigger_animation::<RightSprite>.run_if(input_just_pressed(KeyCode::ArrowRight)),
            )
            .run();

        Self { app }
    }
}

#[derive(Resource)]
struct KeyEventSender(Sender<KeyboardInput>);

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

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
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
