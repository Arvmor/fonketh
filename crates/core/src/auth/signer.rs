use crate::auth::SignedMessage;
use crate::prelude::Serialize;
use game_contract::prelude::{Signature, Signer, keccak256};

/// TxSigner
///
/// Responsible for signing transactions before sending them through channels
pub struct TxSigner<Tx, S> {
    tx: Tx,
    signer: S,
}

impl<Tx, S> TxSigner<Tx, S>
where
    S: Signer,
{
    /// Creates a new TxSigner
    pub fn new(tx: Tx, signer: S) -> Self {
        Self { tx, signer }
    }

    /// Signs and sends the message
    pub async fn send_signed<M>(&self, message: M) -> anyhow::Result<()>
    where
        M: Serialize,
        Tx: TxTrait<Message = SignedMessage<M, Signature>>,
    {
        let hash = keccak256(serde_json::to_vec(&message)?);
        let signed = self.signer.sign_hash(&hash).await?;
        let signed_message = SignedMessage::new(message, signed);

        // Broadcast the signed message
        self.tx.send(signed_message).await
    }
}

/// Tx Trait
///
/// Responsible for sending messages through channels
#[async_trait::async_trait]
pub trait TxTrait {
    type Message;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
impl<M> TxTrait for tokio::sync::mpsc::Sender<M>
where
    M: Send + Sync + 'static,
{
    type Message = M;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()> {
        self.send(message).await.map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl<M> TxTrait for std::sync::mpsc::Sender<M>
where
    M: Send + Sync + 'static,
{
    type Message = M;

    async fn send(&self, message: Self::Message) -> anyhow::Result<()> {
        self.send(message).map_err(Into::into)
    }
}
