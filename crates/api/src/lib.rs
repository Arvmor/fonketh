use actix_web::web;
use actix_web::{App, HttpServer};
use game_primitives::WorldState;
use serde::Serialize;

mod health_status;
mod utils;
mod world_status;

use health_status::HealthStatus;
use world_status::WorldStatus;

/// Api Server
///
/// Responsible for running the API server
pub struct ApiServer<W: WorldState> {
    world: web::ThinData<W>,
}

impl<W, M, P> ApiServer<W>
where
    W: WorldState<Message = M, Player = P> + Clone + Send + Sync + 'static,
    M: Serialize + 'static,
    P: Serialize + 'static,
{
    /// Creates a new API server that holds resources
    pub fn new(world: W) -> Self {
        let world = web::ThinData(world);
        Self { world }
    }

    /// Runs the API server
    pub async fn run(self) {
        let app = move || {
            App::new()
                .app_data(self.world.clone())
                // Health Check Endpoints
                .service(web::resource("/").to(HealthStatus::index))
                .service(web::resource("/health").to(HealthStatus::health_check))
                // World Status Endpoints
                .service(web::resource("/mine").to(WorldStatus::mined_batch::<W>))
                .service(web::resource("/players").to(WorldStatus::players::<W, P>))
                .service(web::resource("/chat").to(WorldStatus::chat_messages::<W, M>))
        };

        HttpServer::new(app)
            .bind("0.0.0.0:8080")
            .expect("Failed to bind to port 8080")
            .run()
            .await
            .expect("Failed to run server");
    }
}
