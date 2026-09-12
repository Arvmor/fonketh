use serde::ser::{Error, SerializeStruct};
use std::fmt::{Display, Formatter, Result};
use std::time::{Duration, Instant};

/// Chat Entry
///
/// Structured, render-friendly view of a chat message.
/// Lets user interfaces show the sender, body and age separately
/// instead of parsing the [`Display`] output.
pub trait ChatEntry {
    /// Stable identifier of the sender (e.g. the address), for comparisons
    fn sender_id(&self) -> &str;
    /// Display name of the sender: the resolved name if known, else the identifier
    fn sender(&self) -> &str;
    /// The message body
    fn content(&self) -> &str;
    /// How long ago the message was received
    fn age(&self) -> Duration;
}

/// Chat Message
///
/// Used to represent a chat message
/// Sent by a player to the world
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// The player's identifier (address)
    pub identifier: String,
    /// The player's resolved display name (e.g. ENS), if any
    pub name: Option<String>,
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

        let mut s = serializer.serialize_struct("ChatMessage", 4)?;
        s.serialize_field("identifier", &self.identifier)?;
        s.serialize_field("name", &self.name)?;
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
            name: None,
            message,
            timestamp,
        }
    }

    /// Attaches a resolved display name (e.g. ENS) to the message
    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name.filter(|n| !n.is_empty());
        self
    }
}

impl ChatEntry for ChatMessage {
    fn sender_id(&self) -> &str {
        &self.identifier
    }

    fn sender(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.identifier)
    }

    fn content(&self) -> &str {
        &self.message
    }

    fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

impl Display for ChatMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {} | {}s ago",
            self.sender(),
            self.message,
            self.timestamp.elapsed().as_secs()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sender_falls_back_to_identifier() {
        let plain = ChatMessage::new("0xabc".into(), "gm".into());
        assert_eq!(plain.sender_id(), "0xabc");
        assert_eq!(plain.sender(), "0xabc");

        let named =
            ChatMessage::new("0xabc".into(), "gm".into()).with_name(Some("vitalik.eth".into()));
        assert_eq!(named.sender_id(), "0xabc");
        assert_eq!(named.sender(), "vitalik.eth");
        assert!(named.to_string().starts_with("vitalik.eth: gm"));

        let empty = ChatMessage::new("0xabc".into(), "gm".into()).with_name(Some(String::new()));
        assert_eq!(empty.sender(), "0xabc");
    }
}
