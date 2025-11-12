use actix_web::web::{self, ThinData};
use actix_web::{App, HttpServer};
use game_primitives::WorldState;

use crate::mine::MineData;

mod mine;

pub struct ApiServer<W: WorldState> {
    world: ThinData<MineData<W>>,
}

impl<W: WorldState + 'static + Clone + Send + Sync> ApiServer<W> {
    pub fn new(world: W) -> Self {
        let world = ThinData(MineData::new(world));
        Self { world }
    }

    pub async fn run(self) {
        HttpServer::new(move || {
            App::new()
                .app_data(self.world.clone())
                .service(web::resource("/mine").to(mine::mine::<W>))
        })
        .bind("0.0.0.0:8080")
        .expect("Failed to bind to port 8080")
        .run()
        .await
        .expect("Failed to run server");
    }
}
