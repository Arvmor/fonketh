use crate::prelude::*;
use alloy::{primitives::keccak256, sol, sol_types::SolValue};

sol!(
    #[sol(rpc)]
    Rewarder,
    "../../contracts/rewarder.json"
);

/// Miner
///
/// Responsible for mining the nonce
#[derive(Debug)]
pub struct Miner {
    pub(crate) factory: Address,
    pub(crate) address: Address,
    pub(crate) salt: U256,
    pub(crate) init_hash: B256,
    pub(crate) difficulty: Address,
}

impl Miner {
    /// Creates a new miner
    pub fn new(
        factory: Address,
        address: Address,
        salt: impl TryInto<U256>,
        init_hash: B256,
        difficulty: Address,
    ) -> Self {
        let salt = salt.try_into().unwrap_or_default();

        Self {
            factory,
            address,
            salt,
            init_hash,
            difficulty,
        }
    }

    /// Mines a new address
    fn mine(&self, address: Address, nonce: U256, init_hash: B256) -> anyhow::Result<()> {
        // keccak256(abi.encodePacked(nonce, minerAddress));
        let salt = keccak256((nonce, address).abi_encode_packed());
        let mined = self.factory.create2(salt, init_hash);

        // Check against the network difficulty
        if mined > self.difficulty {
            return Err(anyhow::anyhow!("Not passed the network difficulty"));
        }

        Ok(())
    }

    /// Run Miner
    pub fn run(&mut self) -> anyhow::Result<(Address, U256)> {
        // Increment the nonce
        self.salt += U256::ONE;

        // If mined, return the miner address and nonce
        self.mine(self.address, self.salt, self.init_hash)?;
        info!("Mined address: {} with salt: {}", self.address, self.salt);

        Ok((self.address, self.salt))
    }

    /// Verify mined address
    pub fn verify(&self, address: Address, nonce: U256) -> anyhow::Result<()> {
        self.mine(address, nonce, self.init_hash)
    }
}
