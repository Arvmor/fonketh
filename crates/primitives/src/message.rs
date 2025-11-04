use std::fmt::{Display, Formatter, Result};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub identifier: String,
    pub message: String,
    pub timestamp: Instant,
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
