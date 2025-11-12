use actix_web::web::{Data, ThinData};
use actix_web::{HttpResponse, Responder};
use game_primitives::WorldState;

/// Mine data
///
/// Used to store the mine data
#[derive(Clone)]
pub struct MineData<W: WorldState> {
    world: W,
}

impl<W: WorldState> MineData<W> {
    pub fn new(world: W) -> Self {
        Self { world }
    }
}

// #[get("/mine")]
pub async fn mine<W: WorldState>(data: ThinData<MineData<W>>) -> impl Responder {
    let batch = data.world.get_mining_batch();
    HttpResponse::Ok().json(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_primitives::{Identifier, Player, Position};
    use std::{
        collections::{HashMap, HashSet},
        sync::Arc,
    };

    struct WorldStateMock {
        pub batch: HashSet<()>,
    }

    impl Identifier for WorldStateMock {
        type Id = ();

        fn identifier(&self) -> Self::Id {}
    }

    struct PlayerMock {}

    impl Identifier for PlayerMock {
        type Id = ();

        fn identifier(&self) -> Self::Id {}
    }

    #[derive(Debug, Clone)]
    struct PositionMock {}

    impl Position for PositionMock {
        type Unit = ();

        fn new(_: Self::Unit, _: Self::Unit) -> Self {
            Self {}
        }

        fn x(&self) -> f64 {
            0.0
        }

        fn y(&self) -> f64 {
            0.0
        }
    }

    impl Player for PlayerMock {
        type Position = PositionMock;

        fn position(&self) -> Self::Position {
            PositionMock {}
        }
    }

    impl WorldState for WorldStateMock {
        type Message = u8;
        type MiningBatch = ();
        type Player = PlayerMock;

        fn exit_status(&self) -> Arc<game_primitives::ExitStatus> {
            Arc::new(game_primitives::ExitStatus::default())
        }

        fn get_all_players(&self) -> HashMap<Self::Id, Self::Player> {
            HashMap::new()
        }

        fn get_mining_rewards_count(&self) -> u32 {
            0
        }

        fn get_mining_batch(&self) -> HashSet<Self::MiningBatch> {
            self.batch.clone()
        }

        fn get_chat_messages(&self) -> Vec<Self::Message> {
            Vec::new()
        }
    }

    #[test]
    fn test_mine_data() {
        let batch = WorldStateMock {
            batch: HashSet::new(),
        };
        MineData::new(batch);
    }
}
