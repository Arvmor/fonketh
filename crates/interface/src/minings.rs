use crate::prelude::*;
use bevy::prelude::*;

/// System to track mining events and update the counter
pub fn track_mining_events<W: WorldState + Sync + Send + 'static>(
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
pub fn update_status_bar(
    mining_rewards: Res<MiningRewards>,
    mut text_query: Query<&mut Text, With<StatusBar>>,
) {
    for mut text in text_query.iter_mut() {
        *text = Text::new(format!("Mining Rewards: {}", mining_rewards.count));
    }
}
