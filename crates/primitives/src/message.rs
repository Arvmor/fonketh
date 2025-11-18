use serde::ser::{Error, SerializeStruct};
use std::fmt::{Display, Formatter, Result};
use std::time::Instant;

/// Chat Message
///
/// Used to represent a chat message
/// Sent by a player to the world
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// The player's identifier
    pub identifier: String,
    /// The message content
    pub message: String,
    /// The timestamp of the message
    pub timestamp: Instant,
}

impl serde::Serialize for ChatMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let epoch = std::time::UNIX_EPOCH.elapsed().map_err(Error::custom)?;
        let timestamp = epoch - self.timestamp.elapsed();

        let mut s = serializer.serialize_struct("ChatMessage", 3)?;
        s.serialize_field("identifier", &self.identifier)?;
        s.serialize_field("message", &self.message)?;
        s.serialize_field("timestamp", &timestamp.as_secs())?;
        s.end()
    }
}

impl ChatMessage {
    /// Creates a new chat message
    pub fn new(identifier: String, message: String) -> Self {
        let timestamp = Instant::now();

        Self {
            identifier,
            message,
            timestamp,
        }
    }
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {} | {}s ago",
            self.identifier,
            self.message,
            self.timestamp.elapsed().as_secs()
        )
    }
}
