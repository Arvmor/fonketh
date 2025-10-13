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

/// Trait for DB Game
pub trait DB {
    type Client;

    /// returns the client
    fn client(&self) -> &Self::Client;
}

impl DB for GameDB {
    type Client = sqlx::PgPool;

    fn client(&self) -> &Self::Client {
        &self.client
    }
}
