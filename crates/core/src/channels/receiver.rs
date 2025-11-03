use crate::channels::message::SignedMessage;

/// Signed Receiver
///
/// Used to receive signed messages from a channel
/// Verifies the signature of the message
pub trait SignedReceiver: Receiver {
    /// Receive a signed message from the channel and verify the signature
    fn receive_signed(&mut self) -> anyhow::Result<Option<Self::Message>> {
        let message = self.try_receive()?;

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
pub trait Receiver {
    type Message: SignedMessage;

    fn try_receive(&mut self) -> anyhow::Result<Option<Self::Message>>;
}

impl<T> Receiver for std::sync::mpsc::Receiver<T>
where
    T: SignedMessage,
{
    type Message = T;

    fn try_receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        match self.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T> Receiver for tokio::sync::mpsc::Receiver<T>
where
    T: SignedMessage,
{
    type Message = T;

    fn try_receive(&mut self) -> anyhow::Result<Option<Self::Message>> {
        match self.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
