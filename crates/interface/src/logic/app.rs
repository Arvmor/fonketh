use crate::chat::{render_chat_input, render_chat_log};
use crate::hud::{hide_hud, hide_toasts, setup_hud, show_hud};
use crate::input::route_keyboard;
use crate::menu::{handle_menu_pointer, spawn_main_menu, spawn_pause_menu, style_menu_buttons};
use crate::movements::{
    execute_animations, follow_main_player_with_camera, handle_idle_transitions,
    track_network_movements,
};
use crate::prelude::*;
use crate::stats::{track_mining_stats, track_population, update_stat_values};
use crate::toast::{ToastRequest, show_toasts, tick_toasts};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use game_primitives::events::GameEvent;
use game_primitives::{Identifier, Player, Position, WorldState};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::mpsc::Sender;

/// Window title
const WINDOW_TITLE: &str = "Fonketh";
/// Initial window size (physical pixels)
const WINDOW_SIZE: (u32, u32) = (1280, 800);

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
        let app = Self::build(channel, world).run();

        Self { app }
    }

    /// Builds the Bevy app without running it
    ///
    /// Lets callers (tests, previews) add their own plugins and systems
    /// before calling [`App::run`].
    pub fn build<W, P, I, F, Po>(channel: Sender<GameEvent<F, Po>>, world: W) -> App
    where
        F: Send + Sync + 'static,
        Po: Position<Unit = i32> + Send + Sync + 'static,
        W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
        P: Identifier<Id = I> + Player + Sync + Send + 'static,
        I: Hash + Eq + Clone + Sync + Send + Display + 'static,
    {
        // Config plugins
        let image_plugin = ImagePlugin::default_nearest();
        let assets_root = AssetsRoot::locate();
        info!("Loading assets from {}", assets_root.0.display());
        let asset_plugin = AssetPlugin {
            file_path: assets_root.0.to_string_lossy().into_owned(),
            ..Default::default()
        };
        let window_plugin = WindowPlugin {
            primary_window: Some(Window {
                title: WINDOW_TITLE.to_string(),
                resolution: WindowResolution::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
                ..default()
            }),
            ..default()
        };

        let mut app = App::new();
        app
            // Channel to pass Events to core
            .insert_resource(KeyEventSender(channel))
            .insert_resource(assets_root)
            .insert_resource(WorldStateResource(world))
            .insert_resource(SpawnedPlayers::<P>::default())
            .insert_resource(PlayerStates::<P>::default())
            .insert_resource(MiningStats::default())
            .insert_resource(Population::default())
            .insert_resource(ChatInputText::default())
            .insert_resource(ChatLogState::default())
            .insert_resource(MenuSelection::default())
            // prevents blurry sprites
            .add_plugins(
                DefaultPlugins
                    .set(image_plugin)
                    .set(asset_plugin)
                    .set(window_plugin),
            )
            .init_state::<Screen>()
            .add_message::<ToastRequest>()
            // Startup systems
            .add_systems(Startup, (setup, setup_hud::<W>))
            // Screen transitions
            .add_systems(OnEnter(Screen::Menu), (spawn_main_menu::<W>, hide_hud))
            .add_systems(OnEnter(Screen::Playing), show_hud)
            .add_systems(OnEnter(Screen::Paused), (spawn_pause_menu, hide_toasts))
            // Input and lifecycle
            .add_systems(
                Update,
                (route_keyboard::<F, Po>, check_shutdown_conditions::<W>),
            )
            // World rendering
            .add_systems(
                Update,
                (
                    spawn_new_players::<W, P, I>,
                    despawn_quit_players::<W, P, I>,
                    track_network_movements::<W, P, I>,
                    handle_idle_transitions::<W, P, I>,
                    execute_animations::<W, P, I>,
                    follow_main_player_with_camera,
                ),
            )
            // HUD
            .add_systems(
                Update,
                (
                    track_mining_stats::<W>,
                    track_population::<W>,
                    update_stat_values,
                    render_chat_log::<W>,
                    render_chat_input,
                    show_toasts,
                    tick_toasts,
                ),
            )
            // Menus
            .add_systems(
                Update,
                (handle_menu_pointer::<F, Po>, style_menu_buttons)
                    .run_if(not(in_state(Screen::Playing))),
            );

        app
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Spawn the grass background
    let image = asset_server.load("assets/textures/background/full.png");
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
    assets_root: Res<AssetsRoot>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + Display + 'static,
{
    // Textures directory holding `characters/` and `hats/`, plus the plain fallback sheet
    let textures = assets_root.textures();
    let fallback = textures.join("characters").join("gabe-idle-run.png");
    let local_player_id = world_state.0.identifier();

    for (peer_id, character) in world_state.0.get_all_players() {
        // If the player has already been spawned, skip
        if !spawned_players.spawned.insert(peer_id.clone()) {
            continue;
        }

        // Compose the sprite sheet (character, hat and hair color) from the player's id
        #[cfg(feature = "custom_sprites")]
        let path = game_sprite::SpriteImage::from_identifier(&textures, peer_id.to_string())
            .unwrap_or_else(|e| {
                error!("Failed to compose sprite image: {e}");
                fallback.clone()
            });
        #[cfg(not(feature = "custom_sprites"))]
        let path = fallback.clone();

        // Load the sprite sheet using the `AssetServer`, relative to the assets root
        let image = asset_server.load(assets_root.asset_path(&path));
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

fn despawn_quit_players<W, P, I>(
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    mut spawned_players: ResMut<SpawnedPlayers<P>>,
    mut player_states: ResMut<PlayerStates<P>>,
    player_query: Query<(Entity, &PlayerEntity<P>)>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + Display + 'static,
{
    let all_players = world_state.0.get_all_players();

    for (entity, player_entity) in player_query.iter() {
        if !all_players.contains_key(&player_entity.peer_id) {
            info!("Despawning player entity: {}", player_entity.peer_id);
            commands.entity(entity).despawn();
            spawned_players.spawned.remove(&player_entity.peer_id);
            player_states.players.remove(&player_entity.peer_id);
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
