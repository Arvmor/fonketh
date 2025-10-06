use crate::prelude::*;
use alloy_primitives::{Address, B256, U256, address};

/// Miner
///
/// Responsible for mining the nonce
#[derive(Debug)]
pub struct Miner {
    pub(crate) mined: (Address, B256),
    pub(crate) salt: U256,
    pub(crate) init_hash: B256,
}

impl Miner {
    /// Mining difficulty (Number of leading zeros in the hash)
    pub const MINING_DIFFICULTY: usize = 2;

    /// Mining Factory Address
    pub const MINING_FACTORY: Address = Address::ZERO;

    /// Creates a new miner
    pub fn new(salt: impl TryInto<U256>, init_hash: B256) -> Self {
        let salt = salt.try_into().unwrap_or_default();
        let mined = (
            address!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
            B256::ZERO,
        );

        Self {
            mined,
            salt,
            init_hash,
        }
    }

    /// Mines a new address
    pub fn mine(&mut self, salt: U256, init_hash: &B256) -> bool {
        let salt = B256::from(salt);
        let mined = Self::MINING_FACTORY.create2(salt, init_hash);

        // Store the best mined address and nonce
        if mined.iter().take(Self::MINING_DIFFICULTY).all(|x| *x == 0) && mined < self.mined.0 {
            info!("Mined address: {mined} with salt: {salt}");
            self.mined = (mined, salt);
            return true;
        }

        false
    }

    /// Run Miner
    pub fn run(&mut self) -> Option<(Address, B256)> {
        let init_hash = self.init_hash;
        self.salt += U256::ONE;

        self.mine(self.salt, &init_hash).then_some(self.mined)
    }
}
