use crate::prelude::*;

/// Player State Struct
#[derive(Debug, Default, Type)]
pub struct Player {
    pub player_id: String,
    pub state: Json<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

impl Player {
    /// Creates a new player state
    pub fn new(
        player_id: String,
        state: serde_json::Value,
        created_at: Option<DateTime<Utc>>,
    ) -> Self {
        let state = Json(state);

        Self {
            player_id,
            state,
            created_at,
        }
    }
}

/// Trait for State DB
#[async_trait]
pub trait StateDB {
    /// Record player state
    async fn record_player_states(&self, player_states: &[Player]) -> anyhow::Result<()>;
    /// Remove player state
    async fn remove_player_state(&self, player_id: &str) -> anyhow::Result<()>;
    /// Get player state
    async fn get_player_state(&self, player_id: &str) -> anyhow::Result<Player>;
    /// Get all player states
    async fn get_player_states(&self) -> anyhow::Result<Vec<Player>>;
}

#[async_trait]
impl<T> StateDB for T
where
    T: DB + Send + Sync,
{
    async fn record_player_states(&self, player_states: &[Player]) -> anyhow::Result<()> {
        let result = sqlx::query!(
            r#"INSERT INTO players (player_id, state, created_at)
            SELECT * FROM UNNEST($1::player[])
            ON CONFLICT (player_id) DO UPDATE
            SET state = EXCLUDED.state"#,
            player_states as &[Player]
        )
        .execute(self.client())
        .await?;

        // Sanity check
        let rows = result.rows_affected();
        let total = player_states.len() as u64;
        if rows != total {
            return Err(anyhow::anyhow!("Failed to record states {rows}/{total}"));
        }

        Ok(())
    }

    async fn remove_player_state(&self, player_id: &str) -> anyhow::Result<()> {
        let result = sqlx::query!(r#"DELETE FROM players WHERE player_id = $1"#, player_id)
            .execute(self.client())
            .await?;

        // Sanity check
        let rows = result.rows_affected();
        if rows != 1 {
            return Err(anyhow::anyhow!("Failed to remove state {rows}/1"));
        }

        Ok(())
    }

    async fn get_player_state(&self, player_id: &str) -> anyhow::Result<Player> {
        let result = sqlx::query_as!(
            Player,
            r#"SELECT * FROM players WHERE player_id = $1"#,
            player_id
        )
        .fetch_one(self.client())
        .await?;

        Ok(result)
    }

    async fn get_player_states(&self) -> anyhow::Result<Vec<Player>> {
        let result = sqlx::query_as!(Player, r#"SELECT * FROM players"#)
            .fetch_all(self.client())
            .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::GameDB;

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_record_player_state(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);

        // Create a player state
        let state = json!({ "x": 0, "y": 0 });
        let player_state = Player::new("test".to_string(), state.clone(), None);

        // Record the player state
        db.record_player_states(&[player_state]).await?;

        // Get the player state
        let result = db.get_player_state("test").await?;
        assert_eq!(result.player_id, "test");
        assert_eq!(result.state.0, state);

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_remove_player_state(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);

        // Create a player state
        let state = json!({ "x": 0, "y": 0 });
        let player_state = Player::new("test".to_string(), state.clone(), None);

        // Record the player state
        db.record_player_states(&[player_state]).await?;

        // Get the player state
        let result = db.get_player_state("test").await?;
        assert_eq!(result.state.0, state);
        assert_eq!(result.player_id, "test");

        // Remove the player state
        db.remove_player_state("test").await?;

        // Get the player state
        let result = db.get_player_state("test").await;
        assert!(result.is_err());

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_get_player_states(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);

        // Create a player state
        let state = json!({ "x": 0, "y": 0 });
        let player_state = Player::new("test".to_string(), state.clone(), None);

        let state2 = json!({ "x": 1, "y": 1 });
        let player_state2 = Player::new("test2".to_string(), state2.clone(), None);

        // Record the player state
        db.record_player_states(&[player_state, player_state2])
            .await?;

        // Get the player states
        let result = db.get_player_states().await?;
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].state.0, state);
        assert_eq!(result[0].player_id, "test");
        assert_eq!(result[1].state.0, state2);
        assert_eq!(result[1].player_id, "test2");

        Ok(())
    }
}
