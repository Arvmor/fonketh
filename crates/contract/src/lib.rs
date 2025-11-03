pub mod ens;
pub mod miner;

/// Common Types
pub mod prelude {
    pub use alloy::primitives::{Address, B256, U256, address, keccak256};
    pub use alloy::signers::{Signature, Signer, local::LocalSigner};
    pub use tracing::{debug, error, info, trace, warn};
}

use crate::ens::EnsRegistry::EnsRegistryInstance;
use crate::miner::{Miner, Rewarder};
use alloy::{
    primitives::{Address, address},
    providers::{DynProvider, Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};

/// Rewarder Client
///
/// Responsible for interacting with the Rewarder contract
#[derive(Debug)]
pub struct RewarderClient {
    pub provider: DynProvider,
    pub ens: EnsRegistryInstance<DynProvider>,
    pub contract: Rewarder::RewarderInstance<DynProvider>,
    pub miner: Miner,
    pub wallet: PrivateKeySigner,
}

impl RewarderClient {
    /// Rewarder Contract Address BASE
    pub const ADDRESS: Address = address!("0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459");
    pub const ENS_ADDRESS: Address = address!("0x0000000000d8e504002cc26e3ec46d81971c1664");

    /// Creates a new Rewarder client
    pub async fn new(url: &str, private_key: &[u8], chain_id: u64) -> anyhow::Result<Self> {
        let wallet = PrivateKeySigner::from_slice(private_key)?;
        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .with_chain_id(chain_id)
            .connect_http(url.parse()?)
            .erased();

        // Get ENS registry
        let ens = ens::EnsRegistry::new(Self::ENS_ADDRESS, provider.clone());

        // Get network difficulty
        let contract = Rewarder::new(Self::ADDRESS, provider.clone());
        let difficulty = contract.difficulty().call().await?;
        let init_hash = contract.initHash().call().await?;

        // Create the miner instance
        let miner = Miner::new(Self::ADDRESS, wallet.address(), 0, init_hash, difficulty);

        Ok(Self {
            ens,
            contract,
            provider,
            miner,
            wallet,
        })
    }
}

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
