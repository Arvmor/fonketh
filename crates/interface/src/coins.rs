use crate::prelude::*;
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
