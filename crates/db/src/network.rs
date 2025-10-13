use crate::prelude::*;

/// Node Struct
///
/// matches the nodes table
#[derive(Debug, Default, Type)]
pub struct Node {
    pub peer_id: String,
    pub ip: String,
    pub created_at: Option<DateTime<Utc>>,
}

impl Node {
    /// Creates a new node
    pub fn new(peer_id: String, ip: String, created_at: Option<DateTime<Utc>>) -> Self {
        Self {
            peer_id,
            ip,
            created_at,
        }
    }
}

/// Network DB Trait
///
/// Used to record, remove, get, and get all nodes
#[async_trait]
pub trait NetworkDB {
    /// Records a node
    async fn record_nodes(&self, nodes: &[Node]) -> anyhow::Result<()>;

    /// Removes a node
    async fn remove_node(&self, peer_id: &str) -> anyhow::Result<()>;

    /// Gets a node
    async fn get_node(&self, peer_id: &str) -> anyhow::Result<Node>;

    /// Gets all nodes
    async fn get_nodes(&self) -> anyhow::Result<Vec<Node>>;
}

#[async_trait]
impl<T> NetworkDB for T
where
    T: DB + Send + Sync,
{
    async fn get_node(&self, peer_id: &str) -> anyhow::Result<Node> {
        let result = sqlx::query_as!(Node, r#"SELECT * FROM nodes WHERE peer_id = $1"#, peer_id)
            .fetch_one(self.client())
            .await?;

        Ok(result)
    }

    async fn get_nodes(&self) -> anyhow::Result<Vec<Node>> {
        let result = sqlx::query_as!(Node, r#"SELECT * FROM nodes"#)
            .fetch_all(self.client())
            .await?;

        Ok(result)
    }

    async fn record_nodes(&self, nodes: &[Node]) -> anyhow::Result<()> {
        let result = sqlx::query!(
            r#"INSERT INTO nodes (peer_id, ip, created_at)
            SELECT * FROM UNNEST($1::node[])
            ON CONFLICT (peer_id) DO UPDATE
            SET ip = EXCLUDED.ip"#,
            nodes as &[Node]
        )
        .execute(self.client())
        .await?;

        // Sanity check
        let rows = result.rows_affected();
        let total = nodes.len() as u64;
        if rows != total {
            return Err(anyhow::anyhow!("Failed to record nodes {rows}/{total}"));
        }

        Ok(())
    }

    async fn remove_node(&self, peer_id: &str) -> anyhow::Result<()> {
        let result = sqlx::query!(r#"DELETE FROM nodes WHERE peer_id = $1"#, peer_id)
            .execute(self.client())
            .await?;

        // Sanity check
        let rows = result.rows_affected();
        if rows != 1 {
            return Err(anyhow::anyhow!("Failed to remove node {rows}/1"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameDB;

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_record_node(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);
        let node = Node::new("test".to_string(), "127.0.0.1".to_string(), None);

        // Record the node
        db.record_nodes(&[node]).await?;

        // Get the node
        let result = db.get_node("test").await?;
        assert_eq!(result.peer_id, "test");

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_add_nodes(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);
        let nodes = vec![
            Node::new("test1".to_string(), "127.0.0.1".to_string(), None),
            Node::new("test2".to_string(), "127.0.0.2".to_string(), None),
        ];

        // Add the nodes
        db.record_nodes(&nodes).await?;

        // Get the nodes
        let result = db.get_nodes().await?;
        assert_eq!(result.len(), nodes.len());

        Ok(())
    }

    #[sqlx::test(migrations = "../../migrations")]
    async fn test_remove_node(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let db = GameDB::from(pool);
        let node = Node::new("test".to_string(), "127.0.0.1".to_string(), None);

        // Add the node
        db.record_nodes(&[node]).await?;

        // Get the node
        let result = db.get_node("test").await?;
        assert_eq!(result.peer_id, "test");

        // Remove the node
        db.remove_node("test").await?;

        // Get the node
        let result = db.get_node("test").await;
        assert!(result.is_err());

        Ok(())
    }
}
