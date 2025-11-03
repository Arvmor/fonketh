use crate::channels::message::SignedMessage;
use async_trait::async_trait;

/// Signed Receiver
///
/// Used to receive signed messages from a channel
/// Verifies the signature of the message
#[async_trait]
pub trait SignedReceiver: Receiver {
    /// Receive a signed message from the channel and verify the signature
    async fn receive_signed(&self) -> anyhow::Result<Option<Self::Message>> {
        let message = self.try_receive().await?;

        // Verify the signature
        if let Some(signature) = &message {
            signature.verify()?;
        }

        Ok(message)
    }
}

/// Receiver
///
/// Used to receive messages from a channel
#[async_trait]
pub trait Receiver {
    type Message: SignedMessage;

    async fn receive(&self) -> anyhow::Result<Self::Message>;
    async fn try_receive(&self) -> anyhow::Result<Option<Self::Message>>;
}
