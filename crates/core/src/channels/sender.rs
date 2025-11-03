use crate::channels::SignedMessage;
use async_trait::async_trait;

/// Signed Sender
///
/// Used to send signed messages to a channel
/// Signs the message before sending it
#[async_trait]
pub trait SignedSender: Sender
where
    Self::Message: Send,
{
    /// Send a signed message to the channel
    async fn send_signed(&self, mut message: Self::Message) -> anyhow::Result<()> {
        message.sign().await?;
        self.send(message).await?;

        Ok(())
    }
}

/// Sender
///
/// Used to send messages to a channel
#[async_trait]
pub trait Sender {
    type Message: SignedMessage;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()>;
}

#[async_trait]
impl<T> Sender for std::sync::mpsc::Sender<T>
where
    T: SignedMessage + Send + Sync + 'static,
{
    type Message = T;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()> {
        self.send(message)?;
        Ok(())
    }
}

#[async_trait]
impl<T> Sender for tokio::sync::mpsc::Sender<T>
where
    T: SignedMessage + Send + Sync + 'static,
{
    type Message = T;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()> {
        self.send(message).await?;
        Ok(())
    }
}
