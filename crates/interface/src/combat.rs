use crate::prelude::*;
use bevy::prelude::*;
use game_primitives::{Identifier, Player, Position, WorldState};
use std::hash::Hash;
use std::time::{Duration, Instant};

const PROJECTILE_SPEED: f32 = 600.0;
const PROJECTILE_LIFETIME_SECS: f32 = 1.5;
const SHOOTING_COOLDOWN_MS: u64 = 300;
const PROJECTILE_SIZE: Vec2 = Vec2::new(12.0, 4.0);
const HIT_DISTANCE: f32 = 40.0;

/// Captures Space key presses and sends a PlayerShot event through the core channel
pub fn handle_shooting_input<W, P, I, F, Po>(
    keys: Res<ButtonInput<KeyCode>>,
    sender: Res<KeyEventSender<F, Po>>,
    mut cooldown: ResMut<ShootingCooldown>,
    player_states: Res<PlayerStates<P>>,
    world_state: Res<WorldStateResource<W>>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + 'static,
    F: Send + Sync + 'static,
    Po: Position<Unit = i32> + Send + Sync + 'static,
{
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let ready = cooldown
        .last_shot
        .map_or(true, |t| t.elapsed() >= Duration::from_millis(SHOOTING_COOLDOWN_MS));

    if !ready {
        return;
    }

    let local_id = world_state.0.identifier();

    let facing = player_states
        .players
        .get(&local_id)
        .map_or(FacingDirection::Right, |s| s.facing);

    let (dx, dy): (i32, i32) = match facing {
        FacingDirection::Right => (1, 0),
        FacingDirection::Left => (-1, 0),
        FacingDirection::Up => (0, 1),
        FacingDirection::Down => (0, -1),
    };

    let direction = Po::new(dx, dy);
    if let Err(e) = sender.0.send(game_primitives::events::GameEvent::PlayerShot(direction)) {
        error!("Error sending shot event: {e:?}");
        return;
    }

    cooldown.last_shot = Some(Instant::now());
}

/// Spawns Bevy projectile entities for newly received world-state projectiles
pub fn spawn_projectiles<W, P, I>(
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    mut spawned_ids: ResMut<SpawnedProjectileIds>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + 'static,
{
    for projectile in world_state.0.get_projectiles() {
        if !spawned_ids.ids.insert(projectile.id) {
            continue;
        }

        let origin = Vec3::new(
            projectile.origin_x as f32 * MAGIC_SPEED,
            projectile.origin_y as f32 * MAGIC_SPEED,
            1.0,
        );

        let velocity = Vec2::new(
            projectile.direction_x as f32,
            projectile.direction_y as f32,
        )
        .normalize_or_zero()
            * PROJECTILE_SPEED;

        let angle = velocity.y.atan2(velocity.x);

        commands.spawn((
            Sprite {
                color: Color::srgb(0.0, 1.0, 1.0),
                custom_size: Some(PROJECTILE_SIZE),
                ..default()
            },
            Transform::from_translation(origin).with_rotation(Quat::from_rotation_z(angle)),
            ProjectileEntity { id: projectile.id },
            ProjectileVelocity(velocity),
            ProjectileLifetime(Timer::from_seconds(PROJECTILE_LIFETIME_SECS, TimerMode::Once)),
        ));
    }
}

/// Moves all active projectiles each frame
pub fn move_projectiles(
    time: Res<Time>,
    mut query: Query<(&ProjectileVelocity, &mut Transform), With<ProjectileEntity>>,
) {
    query.iter_mut().for_each(|(vel, mut transform)| {
        transform.translation.x += vel.0.x * time.delta_secs();
        transform.translation.y += vel.0.y * time.delta_secs();
    });
}

/// Checks for projectile-player collisions and applies visual hit feedback
pub fn check_projectile_collisions<W, P, I>(
    mut commands: Commands,
    world_state: Res<WorldStateResource<W>>,
    projectile_query: Query<(Entity, &ProjectileEntity, &Transform)>,
    player_query: Query<(Entity, &PlayerEntity<P>, &Transform)>,
    mut spawned_ids: ResMut<SpawnedProjectileIds>,
) where
    W: WorldState<Id = I, Player = P> + Sync + Send + 'static,
    P: Identifier<Id = I> + Player + Sync + Send + 'static,
    I: Sync + Send + Clone + Hash + Eq + 'static,
{
    let local_id = world_state.0.identifier();

    let local_projectile_ids: std::collections::HashSet<u64> = world_state
        .0
        .get_projectiles()
        .into_iter()
        .filter(|p| p.owner == local_id)
        .map(|p| p.id)
        .collect();

    for (proj_entity, proj_component, proj_transform) in projectile_query.iter() {
        if !local_projectile_ids.contains(&proj_component.id) {
            continue;
        }

        let proj_pos = proj_transform.translation.truncate();

        for (player_entity, player_component, player_transform) in player_query.iter() {
            if player_component.peer_id == local_id {
                continue;
            }

            let player_pos = player_transform.translation.truncate();
            if proj_pos.distance(player_pos) <= HIT_DISTANCE {
                commands.entity(proj_entity).despawn();
                spawned_ids.ids.remove(&proj_component.id);
                commands.entity(player_entity).insert(HitFlash::new());
                break;
            }
        }
    }
}

/// Ticks projectile lifetime timers and despawns expired projectiles
pub fn despawn_expired_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &ProjectileEntity, &mut ProjectileLifetime)>,
    mut spawned_ids: ResMut<SpawnedProjectileIds>,
) {
    query.iter_mut().for_each(|(entity, proj, mut lifetime)| {
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
            spawned_ids.ids.remove(&proj.id);
        }
    });
}

/// Component marking a player as recently hit for visual feedback
#[derive(Component)]
pub struct HitFlash {
    pub timer: Timer,
}

impl HitFlash {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

/// Applies and clears the hit flash visual effect on players
pub fn tick_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    query.iter_mut().for_each(|(entity, mut sprite, mut flash)| {
        flash.timer.tick(time.delta());
        sprite.color = if flash.timer.is_finished() {
            commands.entity(entity).remove::<HitFlash>();
            Color::WHITE
        } else {
            Color::srgb(1.0, 0.2, 0.2)
        };
    });
}
