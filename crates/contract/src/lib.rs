use alloy::{
    primitives::{Address, address},
    providers::{DynProvider, Provider, ProviderBuilder},
    sol,
};

use crate::Rewarder::RewarderInstance;

/// Rewarder Client
///
/// Responsible for interacting with the Rewarder contract
#[derive(Debug)]
pub struct RewarderClient {
    provider: DynProvider,
    contract: RewarderInstance<DynProvider>,
}

impl RewarderClient {
    /// Rewarder Contract Address BASE
    pub const ADDRESS: Address = address!("0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459");

    /// Creates a new Rewarder client
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let provider = ProviderBuilder::new().connect_http(url.parse()?).erased();
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
        let client = RewarderClient::new("https://mainnet.base.org").await?;

        let result = client.contract.REWARD_AMOUNT().call().await?;
        println!("Reward Amount: {result}");
        Ok(())
    }
}
