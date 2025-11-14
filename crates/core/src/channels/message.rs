use crate::BincodeHelper;
use crate::prelude::{Deserialize, GameEventMessage, Serialize, error};
use async_trait::async_trait;
use game_contract::prelude::{Address, Signature, Signer, keccak256};
use game_network::prelude::gossipsub::Message as GossipMessage;

/// Signed Message
///
/// Used to represent a signed message
#[async_trait]
pub trait SignableMessage {
    fn encoded_data(&self) -> anyhow::Result<Vec<u8>>;
    fn address(&self) -> Address;
    fn signature(&self) -> Signature;
    fn signature_mut(&mut self) -> &mut Signature;

    /// Verifies the signature of the message using the provided address
    fn verify(&self) -> anyhow::Result<()> {
        let hash = keccak256(self.encoded_data()?);

        // Recover and Verify
        let signer = self.signature().recover_address_from_prehash(&hash)?;
        if signer != self.address() {
            error!("The signer is not approved");
            return Err(anyhow::anyhow!("The signer is not approved"));
        }

        Ok(())
    }

    /// Signs the message using the provided signer
    async fn sign<S: Signer + Send + Sync>(&mut self, signer: &S) -> anyhow::Result<()> {
        let hash = keccak256(self.encoded_data()?);
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
    fn encoded_data(&self) -> anyhow::Result<Vec<u8>> {
        let packed = (&self.data, &self.address);
        BincodeHelper::encode(&packed)
    }

    fn address(&self) -> Address {
        self.address
    }

    fn signature(&self) -> Signature {
        self.signature
    }

    fn signature_mut(&mut self) -> &mut Signature {
        &mut self.signature
    }
}

impl SignableMessage for GossipMessage {
    fn verify(&self) -> anyhow::Result<()> {
        SignedMessage::<GameEventMessage>::try_from(self)?.verify()
    }

    fn encoded_data(&self) -> anyhow::Result<Vec<u8>> {
        unreachable!()
    }

    fn address(&self) -> Address {
        unreachable!()
    }

    fn signature(&self) -> Signature {
        unreachable!()
    }

    fn signature_mut(&mut self) -> &mut Signature {
        unreachable!()
    }
}

impl<D: for<'de> Deserialize<'de> + Serialize> TryFrom<&GossipMessage> for SignedMessage<D> {
    type Error = anyhow::Error;

    fn try_from(val: &GossipMessage) -> Result<Self, Self::Error> {
        let data = BincodeHelper::decode::<SignedMessage<D>>(&val.data)?;
        Ok(data)
    }
}

impl<T: Serialize> From<SignedMessage<T>> for Vec<u8> {
    fn from(val: SignedMessage<T>) -> Self {
        BincodeHelper::encode(&val).unwrap()
    }
}
