/// Miner Logic
mod mine;

/// Common Types
pub mod prelude {
    pub use alloy::primitives::{Address, B256, U256, address};
    pub use tracing::{debug, error, info, trace, warn};
}

use crate::Rewarder::RewarderInstance;
use alloy::{
    primitives::{Address, B256, address, keccak256},
    providers::{DynProvider, Provider, ProviderBuilder},
    signers::{Signature, Signer, local::PrivateKeySigner},
    sol,
};

/// Rewarder Client
///
/// Responsible for interacting with the Rewarder contract
#[derive(Debug)]
pub struct RewarderClient {
    pub provider: DynProvider,
    pub contract: RewarderInstance<DynProvider>,
    pub miner: mine::Miner,
    pub wallet: PrivateKeySigner,
}

impl RewarderClient {
    /// Rewarder Contract Address BASE
    pub const ADDRESS: Address = address!("0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459");

    /// Creates a new Rewarder client
    pub async fn new(url: &str, private_key: &[u8], chain_id: u64) -> anyhow::Result<Self> {
        let wallet = PrivateKeySigner::from_slice(private_key)?;
        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .with_chain_id(chain_id)
            .connect_http(url.parse()?)
            .erased();

        // Get network difficulty
        let contract = Rewarder::new(Self::ADDRESS, provider.clone());
        let difficulty = contract.difficulty().call().await?;
        let init_hash = contract.initHash().call().await?;

        // Create the miner instance
        let miner = mine::Miner::new(Self::ADDRESS, wallet.address(), 0, init_hash, difficulty);

        Ok(Self {
            contract,
            provider,
            miner,
            wallet,
        })
    }

    /// Hashes a message
    pub fn hash(&self, message: impl AsRef<[u8]>) -> B256 {
        keccak256(message)
    }

    /// Signs a message
    pub async fn create_signature(&self, message: &[u8]) -> anyhow::Result<Signature> {
        let signature = self.wallet.sign_hash(&B256::from_slice(message)).await?;
        Ok(signature)
    }

    /// Verify an event
    pub fn verify_signature(
        &self,
        signature: &Signature,
        address: Address,
        hash: &B256,
    ) -> anyhow::Result<()> {
        let signature = signature.recover_address_from_prehash(hash)?;

        // Verify the signature
        if signature != address {
            tracing::error!("Invalid signer: {signature} != {address}");
            return Err(anyhow::anyhow!("Invalid signer"));
        }

        Ok(())
    }
}

sol!(
    #[sol(rpc)]
    Rewarder,
    "../../contracts/rewarder.json"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> anyhow::Result<()> {
        let client = RewarderClient::new("https://mainnet.base.org", &[1; 32], 8453).await?;

        let result = client.contract.REWARD_AMOUNT().call().await?;
        println!("Reward Amount: {result}");
        Ok(())
    }
}
