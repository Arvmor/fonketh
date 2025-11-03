mod message;
mod receiver;
mod sender;

pub use message::{SignableMessage, SignedMessage};
pub use receiver::{Receiver, SignedReceiver};
pub use sender::{Sender, SignedSender};

#[cfg(test)]
mod tests {
    use super::*;
    use game_contract::prelude::{Address, LocalSigner};

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

    #[tokio::test]
    async fn test_signed_message_tokio() -> anyhow::Result<()> {
        let signer = LocalSigner::random();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // Send message
        let message = SignedMessage::new((), signer.address());
        tx.send_signed(message, &signer).await?;

        // Receive message
        rx.receive_signed()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_malicious_message() -> anyhow::Result<()> {
        let signer = LocalSigner::random();
        let (tx, mut rx) = std::sync::mpsc::channel();

        // Send message
        let message = SignedMessage::new((), Address::ZERO);
        tx.send_signed(message, &signer).await?;

        // Receive message
        assert!(rx.receive_signed().is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_message_sync() -> anyhow::Result<()> {
        const DATA: &[u8] = b"FONKETH";
        let signer = LocalSigner::random();
        let (tx, mut rx) = std::sync::mpsc::channel();

        // Send message
        let message = SignedMessage::new(DATA, signer.address());
        tx.send_signed(message, &signer).await?;

        // Receive message
        let data = rx.receive_signed()?;
        assert_eq!(data.unwrap().data, DATA);
        Ok(())
    }
}
