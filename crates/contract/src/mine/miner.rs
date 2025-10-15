use crate::prelude::*;
use alloy::{primitives::keccak256, sol_types::SolValue};

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
    pub fn mine(&mut self, nonce: U256, init_hash: B256) -> Option<U256> {
        // keccak256(abi.encodePacked(nonce, minerAddress));
        let salt = keccak256((nonce, self.address).abi_encode_packed());
        let mined = self.factory.create2(salt, init_hash);

        // If passed the difficulty, return the nonce
        if mined < self.difficulty {
            info!("Mined address: {mined} with salt: {salt}");
            return Some(nonce);
        }

        None
    }

    /// Run Miner
    pub fn run(&mut self) -> Option<(Address, U256)> {
        // Increment the nonce
        self.salt += U256::ONE;

        // If mined, return the miner address and nonce
        self.mine(self.salt, self.init_hash)
            .map(|n| (self.address, n))
    }
}
