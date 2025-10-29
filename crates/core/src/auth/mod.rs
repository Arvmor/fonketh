mod signer;
mod verifier;

use crate::prelude::{Deserialize, Serialize};

pub use game_contract::prelude::{Signature, Signer};
pub use signer::{TxSigner, TxTrait};
pub use verifier::{RxTrait, RxVerifier};

/// Signed Message Payload
///
/// Contains the message and the signature
#[derive(Debug, Serialize, Deserialize)]
pub struct SignedMessage<M, S> {
    pub message: M,
    pub signature: S,
}

impl<M, S> SignedMessage<M, S> {
    /// Creates a new signed message
    pub fn new(message: M, signature: S) -> Self {
        Self { message, signature }
    }
}

/// Signed Signature Trait
///
/// Responsible for recovering the address from a signed message
pub trait SignedSignature {
    type Address;
    /// Recovers the address from the message
    fn recover_address(&self) -> anyhow::Result<Self::Address>;
}

impl<M> SignedSignature for SignedMessage<M, Signature>
where
    M: AsRef<[u8]>,
{
    type Address = game_contract::prelude::Address;

    fn recover_address(&self) -> anyhow::Result<Self::Address> {
        let address = self.signature.recover_address_from_msg(&self.message)?;
        Ok(address)
    }
}

impl SignedSignature for game_network::prelude::gossipsub::Message {
    type Address = game_contract::prelude::Address;

    fn recover_address(&self) -> anyhow::Result<Self::Address> {
        let signed = serde_json::from_slice::<SignedMessage<Vec<u8>, Signature>>(&self.data)?;
        let address = signed.signature.recover_address_from_msg(&signed.message)?;

        Ok(address)
    }
}
