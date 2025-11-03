use crate::channels::SignableMessage;
use async_trait::async_trait;
use game_contract::prelude::Signer;

/// Signed Sender
///
/// Used to send signed messages to a channel
/// Signs the message before sending it
#[async_trait]
pub trait SignedSender: Sender
where
    Self::Message: SignableMessage + Send,
{
    /// Send a signed message to the channel
    async fn send_signed<S: Signer + Send + Sync>(
        &self,
        mut message: Self::Message,
        signer: &S,
    ) -> anyhow::Result<()> {
        message.sign(signer).await?;
        self.send(message).await?;

        Ok(())
    }
}

impl<T> SignedSender for T
where
    T: Sender,
    T::Message: SignableMessage + Send,
{
}

/// Sender
///
/// Used to send messages to a channel
#[async_trait]
pub trait Sender {
    type Message;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()>;
}

#[async_trait]
impl<T> Sender for std::sync::mpsc::Sender<T>
where
    T: Send + Sync + 'static,
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
    T: Send + Sync + 'static,
{
    type Message = T;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()> {
        self.send(message).await?;
        Ok(())
    }
}
