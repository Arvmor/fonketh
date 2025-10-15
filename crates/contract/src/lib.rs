/// Miner Logic
mod mine;

/// Common Types
pub mod prelude {
    pub use alloy::primitives::{Address, B256, U256, address};
    pub use tracing::{debug, error, info, trace, warn};
}

use crate::Rewarder::RewarderInstance;
use alloy::{
    primitives::{Address, address},
    providers::{DynProvider, Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
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
}

impl RewarderClient {
    /// Rewarder Contract Address BASE
    pub const ADDRESS: Address = address!("0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459");

    /// Creates a new Rewarder client
    pub async fn new(url: &str, private_key: &[u8], chain_id: u64) -> anyhow::Result<Self> {
        let wallet = PrivateKeySigner::from_slice(private_key)?;
        let address = wallet.address();
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .with_chain_id(chain_id)
            .connect_http(url.parse()?)
            .erased();

        // Get network difficulty
        let contract = Rewarder::new(Self::ADDRESS, provider.clone());
        let difficulty = contract.difficulty().call().await?;
        let init_hash = contract.initHash().call().await?;

        // Create the miner instance
        let miner = mine::Miner::new(Self::ADDRESS, address, 0, init_hash, difficulty);

        Ok(Self {
            contract,
            provider,
            miner,
        })
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
