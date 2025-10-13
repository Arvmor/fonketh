/// Persists networking data
pub mod network;

/// Prelude
pub mod prelude {
    pub use crate::DB;
    pub use chrono::{DateTime, Utc};
    pub use sqlx::prelude::Type;
}

/// DB Client
#[derive(Debug, Clone)]
pub struct GameDB {
    client: sqlx::PgPool,
}

impl GameDB {
    /// Creates a new DB client
    ///
    /// Connects to the database using the provided URL
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let client = sqlx::PgPool::connect(url).await?;

        Ok(Self { client })
    }
}

impl From<sqlx::PgPool> for GameDB {
    fn from(client: sqlx::PgPool) -> Self {
        Self { client }
    }
}

/// Trait for DB Game
pub trait DB {
    /// returns the client
    fn client(&self) -> &sqlx::PgPool;
}

impl DB for GameDB {
    fn client(&self) -> &sqlx::PgPool {
        &self.client
    }
}
