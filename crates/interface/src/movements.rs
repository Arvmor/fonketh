use crate::logic::keyboard_events;
use crate::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use std::hash::Hash;
use std::time::Instant;

/// Player state information bundled together for better cache locality
#[derive(Debug, Clone)]
pub struct PlayerStateInfo {
    pub state: CharacterState,
    pub last_movement_time: Instant,
    pub previous_position: (i64, i64),
    pub facing_right: bool,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CharacterState {
    #[default]
    Idle,
    Running,
}

/// Captures keyboard events and sends them to the core channel
pub fn capture_key_events<F, Po>(
    mut evr_keys: MessageReader<KeyboardInput>,
    sender: Res<KeyEventSender<F, Po>>,
) where
    F: Send + Sync + 'static,
    Po: Position<Unit = i32> + Send + Sync + 'static,
{
    for ev in evr_keys.read() {
        info!("Keyboard event: {ev:?}");

        // Send over channel to core
        if let Some(event) = keyboard_events(ev.key_code)
            && let Err(e) = sender.0.send(event)
        {
            error!("Error sending keyboard event: {e:?}");
        }
    }
}

/// Tracks network player movements by comparing current positions with previous positions
pub fn track_network_movements<W, P, I>(
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

/// Combined system to update player positions and execute animations
/// This system handles both position updates and animation frame progression for all players
pub fn execute_animations<W, P, I>(
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

        // Calculate position directly from world position
        // Camera follows the player now, so no offset needed
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
                if !config.frame_timer.is_paused() {
                    config.frame_timer.pause();
                }
            }
            CharacterState::Running => {
                // Resume animation timer if paused
                if config.frame_timer.is_paused() {
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
pub fn handle_idle_transitions<W, P, I>(
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

/// System to update camera position to follow the main player with boundary logic
pub fn follow_main_player_with_camera(
    main_player_query: Query<&Transform, (With<MainPlayer>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    window_query: Query<&Window>,
) {
    // Get the main player's position (using iter to get the first result)
    let Some(player_transform) = main_player_query.iter().next() else {
        return;
    };

    // Get the camera's current position (using iter_mut to get the first result)
    let Some(mut camera_transform) = camera_query.iter_mut().next() else {
        return;
    };

    // Get window size to calculate dynamic boundaries
    let Some(window) = window_query.iter().next() else {
        return;
    };

    // Calculate dynamic camera boundaries based on window size
    // Boundaries are proportional to the window dimensions
    let camera_boundary_x = ((window.width() / 2.) - CAMERA_BOUNDARY_X).max(0.);
    let camera_boundary_y = ((window.height() / 2.) - CAMERA_BOUNDARY_Y).max(0.);

    let player_x = player_transform.translation.x;
    let player_y = player_transform.translation.y;
    let camera_x = camera_transform.translation.x;
    let camera_y = camera_transform.translation.y;

    // Calculate the offset between camera and player
    let offset_x = player_x - camera_x;
    let offset_y = player_y - camera_y;

    // Only move camera if player goes outside the boundary
    let mut new_camera_x = camera_x;
    let mut new_camera_y = camera_y;

    // Check horizontal boundary
    if offset_x > camera_boundary_x {
        new_camera_x = player_x - camera_boundary_x;
    } else if offset_x < -camera_boundary_x {
        new_camera_x = player_x + camera_boundary_x;
    }

    // Check vertical boundary
    if offset_y > camera_boundary_y {
        new_camera_y = player_y - camera_boundary_y;
    } else if offset_y < -camera_boundary_y {
        new_camera_y = player_y + camera_boundary_y;
    }

    // Update camera position
    camera_transform.translation.x = new_camera_x;
    camera_transform.translation.y = new_camera_y;
}
