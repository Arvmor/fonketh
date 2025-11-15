use crate::utils::ResponseAPI;
use actix_web::Responder;
use actix_web::web;
use game_primitives::WorldState;
use serde::Serialize;

/// API Endpoints for the world status
pub struct WorldStatus;

impl WorldStatus {
    /// Responds with latest state of the world.
    pub async fn mined_batch<W: WorldState>(data: web::ThinData<W>) -> impl Responder {
        let batch = data.get_mining_batch();
        ResponseAPI::success(batch)
    }

    /// Responds with players in the world
    pub async fn players<W: WorldState<Player = P>, P: Serialize>(
        data: web::ThinData<W>,
    ) -> impl Responder {
        let players = data.get_all_players().into_values().collect::<Vec<_>>();
        ResponseAPI::success(players)
    }

    /// Responds with chat messages in the world
    pub async fn chat_messages<W: WorldState<Message = M>, M: Serialize>(
        data: web::ThinData<W>,
    ) -> impl Responder {
        let messages = data.get_chat_messages();
        ResponseAPI::success(messages)
    }
}
