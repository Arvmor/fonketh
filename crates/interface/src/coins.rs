use crate::prelude::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

pub fn animate_coin(
    time: Res<Time>,
    mut coin_query: Query<(&mut AnimationConfig, &mut Sprite), With<Coin>>,
) {
    for (mut config, mut sprite) in coin_query.iter_mut() {
        let Some(atlas) = &mut sprite.texture_atlas else {
            continue;
        };

        config.frame_timer.tick(time.delta());

        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                atlas.index = config.first_sprite_index;
            } else {
                atlas.index += 1;
            }

            config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
        }
    }
}

const CLAIM_DISTANCE: f32 = 100.0;

pub fn handle_claim_input(
    mut evr_keys: MessageReader<KeyboardInput>,
    mut claim_key: ResMut<ClaimKeyPressed>,
) {
    for ev in evr_keys.read() {
        if ev.key_code == KeyCode::KeyE {
            claim_key.pressed = true;
        }
    }
}

pub fn claim_coins(
    mut commands: Commands,
    mut claim_key: ResMut<ClaimKeyPressed>,
    main_player_query: Query<&Transform, With<MainPlayer>>,
    coin_query: Query<(Entity, &Transform), With<Coin>>,
) {
    if !claim_key.pressed {
        return;
    }

    claim_key.pressed = false;

    let Some(player_transform) = main_player_query.iter().next() else {
        return;
    };

    let player_pos = player_transform.translation;

    for (coin_entity, coin_transform) in coin_query.iter() {
        let coin_pos = coin_transform.translation;
        let distance = player_pos.distance(coin_pos);

        if distance <= CLAIM_DISTANCE {
            info!("Claiming coin at distance: {}", distance);
            commands.entity(coin_entity).despawn();
        }
    }
}
