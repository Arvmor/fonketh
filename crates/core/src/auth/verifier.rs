use crate::auth::SignedSignature;

/// RxVerifier
///
/// Responsible for verifying transactions after receiving them through channels
pub struct RxVerifier<Rx: RxTrait> {
    rx: Rx,
}

impl<Rx, S> RxVerifier<Rx>
where
    Rx: RxTrait<Message = S>,
    S: SignedSignature,
{
    /// Creates a new RxVerifier
    pub fn new(rx: Rx) -> Self {
        Self { rx }
    }

    /// Verifies the message
    pub async fn receive_verified(&self) -> anyhow::Result<S::Address> {
        let message = self.rx.recv().await?;
        let verified = message.recover_address()?;
        Ok(verified)
    }
}

#[async_trait::async_trait]
impl<Rx> RxTrait for RxVerifier<Rx>
where
    Rx: RxTrait + Send + Sync,
{
    type Message = Rx::Message;

    async fn recv(&self) -> anyhow::Result<Self::Message> {
        self.rx.recv().await
    }

    async fn try_recv(&self) -> anyhow::Result<Option<Self::Message>> {
        self.rx.try_recv().await
    }
}

/// Rx Trait
///
/// Responsible for receiving messages through channels
#[async_trait::async_trait]
pub trait RxTrait {
    type Message;

    async fn recv(&self) -> anyhow::Result<Self::Message>;
    async fn try_recv(&self) -> anyhow::Result<Option<Self::Message>>;
}

#[async_trait::async_trait]
impl<M> RxTrait for tokio::sync::mpsc::Receiver<M>
where
    M: Send + Sync + 'static,
{
    type Message = M;

    async fn recv(&self) -> anyhow::Result<Self::Message> {
        self.recv().await
    }

    async fn try_recv(&self) -> anyhow::Result<Option<Self::Message>> {
        self.try_recv().await
    }
}

// #[async_trait::async_trait]
// impl<M> RxTrait for std::sync::mpsc::Receiver<M>
// where
//     M: Send + Sync + 'static,
// {
//     type Message = M;

//     async fn recv(&self) -> anyhow::Result<Self::Message> {
//         self.recv().map_err(Into::into)
//     }
// }
