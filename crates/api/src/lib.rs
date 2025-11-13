use actix_web::Responder;
use actix_web::web;
use actix_web::{App, HttpServer, get};
use game_primitives::WorldState;
use serde::Serialize;

mod world_status;

/// Index route
///
/// Responds with a simple message
#[get("/")]
async fn index() -> impl Responder {
    "Live Server"
}

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
                .service(index)
                .service(web::resource("/mine").to(world_status::mined_batch::<W>))
                .service(web::resource("/players").to(world_status::players::<W, P>))
                .service(web::resource("/chat").to(world_status::chat_messages::<W, M>))
        };

        HttpServer::new(app)
            .bind("0.0.0.0:8080")
            .expect("Failed to bind to port 8080")
            .run()
            .await
            .expect("Failed to run server");
    }
}
