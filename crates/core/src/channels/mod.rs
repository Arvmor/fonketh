mod message;
mod receiver;
mod sender;

pub use message::{SignableMessage, SignedMessage};
pub use receiver::{Receiver, SignedReceiver};
pub use sender::{Sender, SignedSender};

#[cfg(test)]
mod tests {
    use super::*;
    use game_contract::prelude::LocalSigner;

    #[tokio::test]
    async fn test_signed_message() -> anyhow::Result<()> {
        let signer = LocalSigner::random();
        let (tx, mut rx) = std::sync::mpsc::channel();

        // Send message
        let message = SignedMessage::new((), signer.address());
        tx.send_signed(message, &signer).await?;

        // Receive message
        rx.receive_signed()?;
        Ok(())
    }
}
