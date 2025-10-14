/// Miner Logic
pub mod mine;

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
}

impl RewarderClient {
    /// Rewarder Contract Address BASE
    pub const ADDRESS: Address = address!("0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459");

    /// Creates a new Rewarder client
    pub async fn new(url: &str, private_key: &[u8; 32], chain_id: u64) -> anyhow::Result<Self> {
        let provider = ProviderBuilder::new()
            .wallet(PrivateKeySigner::from_slice(private_key)?)
            .with_chain_id(chain_id)
            .connect_http(url.parse()?)
            .erased();
        let contract = Rewarder::new(Self::ADDRESS, provider.clone());

        Ok(Self { contract, provider })
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
