# Fonketh - P2P Mining Pool

Poc Contract Live on **Base Chain**.

Contract Address at [0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459](https://basescan.org/address/0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459)

## Overview

Fonketh is essentially a **Peer-2-peer mining pool** / **`Gameified PoW Node`**

Allows nodes to join a decentralized mining pool and **compound their mining powers**.

The nodes are rewarded with `$FONK` tokens based on their mining power & contributions.

<video src="./docs/v0.1.0demo.mp4" alt="Fonketh Contract" autoplay loop muted>

## Implementation

The implementation will be based on CoW Protocol's `Solvers` protocol (or any other protocol that supports mining).

Players will be actively processing CoW Protocols solver tasks _(under the hood of the game logic)_,

Gossiping the results to the network and broadcasting the claims on-chain,

Finally, getting rewarded based on their contributions.

<!-- image -->
<img src="./docs/rewards_erc20.png" alt="Fonketh Rewards ERC20" />

-> [Claim Transaction Example](https://basescan.org/tx/0x38a361c7024107052d1c641c45c6273c639ba13cf3c997c6a1d5426dbdaf2370)

### Current Implementation

Currently, the implementation is based on `CREATE2` mining logic.

Players will be mining `CREATE2` addresses based on a given `Network Difficulty`

And gossiping their mined blocks to the network (every 10 blocks across the network).

(As writing this, the difficulty is < `0X000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF`)

```rust
pub fn mine(&mut self, nonce: U256, init_hash: B256) -> Option<U256> {
    // Mine the address
    let salt = keccak256((nonce, self.address).abi_encode_packed());
    let mined = self.factory.create2(salt, init_hash);

    // If passed the network difficulty
    if mined < self.difficulty {
        info!("Mined address: {mined} with salt: {salt}");
        return Some(nonce);
    }

    None
}
```
