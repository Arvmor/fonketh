//! Mining and population statistics, plus the toasts they trigger.

use crate::prelude::*;
use crate::theme::palette;
use crate::toast::ToastRequest;
use bevy::prelude::*;

/// Derives session totals from the world's pending-batch counter
pub fn track_mining_stats<W: WorldState + Sync + Send + 'static>(
    world_state: Res<WorldStateResource<W>>,
    mut stats: ResMut<MiningStats>,
    mut toasts: MessageWriter<ToastRequest>,
) {
    let pending = world_state.0.get_mining_rewards_count();

    match stats.observe(pending) {
        None => {}
        Some(MiningChange::Mined { mined }) => {
            toasts.write(
                ToastRequest::new(
                    if mined == 1 {
                        "Treasure mined".to_string()
                    } else {
                        format!("{mined} treasures mined")
                    },
                    palette::ACCENT,
                )
                .with_detail(format!("{pending}/{CLAIM_BATCH_SIZE} towards next claim")),
            );
        }
        Some(MiningChange::Claimed { .. }) => {
            toasts.write(
                ToastRequest::new("Claim submitted on-chain", palette::SUCCESS)
                    .with_detail(format!("Batch of {CLAIM_BATCH_SIZE} treasures")),
            );
        }
    }
}

/// Watches the player count and toasts on joins and leaves
pub fn track_population<W: WorldState + Sync + Send + 'static>(
    world_state: Res<WorldStateResource<W>>,
    mut population: ResMut<Population>,
    mut toasts: MessageWriter<ToastRequest>,
) {
    let online = world_state.0.get_all_players().len();

    match population.online {
        Some(previous) if previous < online => {
            toasts.write(
                ToastRequest::new("Miner joined", palette::INFO)
                    .with_detail(format!("{online} online")),
            );
        }
        Some(previous) if previous > online => {
            toasts.write(
                ToastRequest::new("Miner left", palette::MUTED)
                    .with_detail(format!("{online} online")),
            );
        }
        _ => {}
    }

    population.online = Some(online);
}

/// Writes the live values into every stat text node
pub fn update_stat_values(
    stats: Res<MiningStats>,
    population: Res<Population>,
    mut values: Query<(&StatValue, &mut Text)>,
    mut fill: Query<&mut Node, With<BatchProgressFill>>,
) {
    for (value, mut text) in values.iter_mut() {
        let rendered = match value {
            StatValue::SessionTotal => stats.session_total.to_string(),
            StatValue::PendingBatch => format!("{}/{CLAIM_BATCH_SIZE}", stats.pending),
            StatValue::Claims => stats.claims.to_string(),
            StatValue::PlayersOnline => population.online.unwrap_or(0).to_string(),
        };
        if text.0 != rendered {
            text.0 = rendered;
        }
    }

    let percent = (stats.pending as f32 / CLAIM_BATCH_SIZE as f32 * 100.0).clamp(0.0, 100.0);
    for mut node in fill.iter_mut() {
        node.width = Val::Percent(percent);
    }
}
