use serde::ser::SerializeStruct;
use std::fmt::{Display, Formatter, Result};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub identifier: String,
    pub message: String,
    pub timestamp: Instant,
}

impl serde::Serialize for ChatMessage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ChatMessage", 3)?;
        s.serialize_field("identifier", &self.identifier)?;
        s.serialize_field("message", &self.message)?;
        s.serialize_field("timestamp", &self.timestamp.elapsed().as_secs())?;
        s.end()
    }
}

impl ChatMessage {
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
