use crate::prelude::{Deserialize, Serialize, error};
use async_trait::async_trait;
use game_contract::prelude::{Address, Signature, Signer, keccak256};

/// Signed Message
///
/// Used to represent a signed message
#[async_trait]
pub trait SignableMessage {
    fn encoded_data(&self) -> Vec<u8>;
    fn address(&self) -> &Address;
    fn signature(&self) -> &Signature;
    fn signature_mut(&mut self) -> &mut Signature;

    /// Verifies the signature of the message using the provided address
    fn verify(&self) -> anyhow::Result<()> {
        let hash = keccak256(self.encoded_data());

        // Recover and Verify
        let signer = self.signature().recover_address_from_prehash(&hash)?;
        if signer != *self.address() {
            error!("The signer is not approved");
            return Err(anyhow::anyhow!("The signer is not approved"));
        }

        Ok(())
    }

    /// Signs the message using the provided signer
    async fn sign<S: Signer + Send + Sync>(&mut self, signer: &S) -> anyhow::Result<()> {
        let hash = keccak256(self.encoded_data());
        *self.signature_mut() = signer.sign_hash(&hash).await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SignedMessage<D: Serialize> {
    pub data: D,
    pub address: Address,
    pub signature: Signature,
}

impl<D: Serialize> SignedMessage<D> {
    pub fn new(data: D, address: Address) -> Self {
        let signature = Signature::new(Default::default(), Default::default(), Default::default());

        Self {
            data,
            address,
            signature,
        }
    }
}

impl<D: Serialize> SignableMessage for SignedMessage<D> {
    fn encoded_data(&self) -> Vec<u8> {
        let packed = (&self.data, &self.address);
        serde_json::to_vec(&packed).unwrap()
    }

    fn address(&self) -> &Address {
        &self.address
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn signature_mut(&mut self) -> &mut Signature {
        &mut self.signature
    }
}

impl SignableMessage for game_network::prelude::gossipsub::Message {
    fn encoded_data(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn address(&self) -> &Address {
        unreachable!()
    }

    fn signature(&self) -> &Signature {
        unreachable!()
    }

    fn signature_mut(&mut self) -> &mut Signature {
        unreachable!()
    }
}

impl<T: Serialize> From<SignedMessage<T>> for Vec<u8> {
    fn from(val: SignedMessage<T>) -> Self {
        serde_json::to_vec(&val).unwrap()
    }
}
