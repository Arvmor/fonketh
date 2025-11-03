use async_trait::async_trait;
use game_contract::prelude::{Address, Signature, Signer, keccak256};

/// Signed Message
///
/// Used to represent a signed message
#[async_trait]
pub trait SignedMessage {
    type Signer: Signer;

    fn data(&self) -> &[u8];
    fn address(&self) -> &Address;
    fn signer(&self) -> &Self::Signer;
    fn signature(&self) -> &Signature;
    fn signature_mut(&mut self) -> &mut Signature;

    /// Verifies the signature of the message using the provided address
    fn verify(&self) -> anyhow::Result<()> {
        let hash = keccak256(self.data());

        // Recover and Verify
        let signer = self.signature().recover_address_from_prehash(&hash)?;
        if signer != *self.address() {
            return Err(anyhow::anyhow!("The signer is not approved"));
        }

        Ok(())
    }

    /// Signs the message using the provided signer
    async fn sign(&mut self) -> anyhow::Result<()> {
        let hash = keccak256(self.data());
        *self.signature_mut() = self.signer().sign_hash(&hash).await?;

        Ok(())
    }
}
