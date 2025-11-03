use async_trait::async_trait;
use game_contract::prelude::{Address, Signature, Signer, SolValue, keccak256};

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

#[derive(Debug, Clone, Copy)]
pub struct SignedMessage<D: SolValue> {
    pub data: D,
    pub address: Address,
    pub signature: Signature,
}

impl<D> SignedMessage<D>
where
    D: SolValue,
{
    pub fn new(data: D, address: Address) -> Self {
        let signature = Signature::new(Default::default(), Default::default(), Default::default());

        Self {
            data,
            address,
            signature,
        }
    }
}

impl<D> SignableMessage for SignedMessage<D>
where
    D: SolValue,
{
    fn encoded_data(&self) -> Vec<u8> {
        (&self.data.abi_encode(), &self.address).abi_encode()
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
