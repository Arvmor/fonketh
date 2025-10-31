# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fonketh is a P2P mining pool / "Gameified PoW Node" built in Rust. Players join a decentralized mining pool to compound their mining power and earn $FONK tokens based on their contributions. The implementation uses CREATE2 mining logic where players mine addresses based on network difficulty and gossip results across a P2P network.

Live PoC Contract on Base Chain: `0xd61e2af6a7c347713c478c4e9fef8fe5a22c5459`

## Build and Run Commands

```bash
# Build the entire workspace
cargo build --release

# Run the application (binary in crates/app)
cargo run --bin app --release

# Run with custom private key file
cargo run --bin app --release path/to/key.txt

# Run with environment variable
PRIVATE_KEY=<hex_private_key> cargo run --bin app --release

# Run tests
cargo test

# Run tests for specific crate
cargo test -p game_contract
cargo test -p game_network
```

## Private Key Management

The app loads a private key in the following priority order:
1. `PRIVATE_KEY` environment variable
2. Command-line argument (path to key file)
3. `./private.key` file (default)
4. Generates a new key and saves to `./private.key` if none found

## Architecture

### Workspace Structure

The codebase is organized as a Cargo workspace with 7 crates:

- **game_app** (`crates/app`): Main binary entry point. Initializes the World with a Character and starts the game loop.

- **game_core** (`crates/core`): Core game logic containing the World state management, player pool, and main event loop. The World orchestrates three main components:
  - Network layer (P2P gossip)
  - Mining contract client (CREATE2 mining)
  - Interface layer (optional, Bevy UI)

- **game_contract** (`crates/contract`): Blockchain interaction layer using Alloy. Contains:
  - `RewarderClient`: Interfaces with the Base chain contract
  - `Miner`: Implements CREATE2 mining logic, attempting to find addresses below network difficulty
  - Contract ABI loaded from `contracts/rewarder.json`

- **game_network** (`crates/network`): P2P networking using libp2p with gossipsub, mDNS discovery, and Kademlia DHT.

- **game_interface** (`crates/interface`): Optional Bevy-based game UI. Enabled via `interface` feature flag in game_core. Handles keyboard input, camera, HUD, chat, and sprite rendering.

- **game_primitives** (`crates/primitives`): Shared types and traits (GameEvent, WorldState, ExitStatus, Identifier).

- **game_sprite** (`crates/sprite`): Character sprite animation logic and asset handling (optional dependency of game_interface).

### Key Architectural Patterns

**Event-Driven World Updates**: The World runs a main loop in `crates/core/src/map/world.rs:130` that:
1. Receives keyboard events from the interface (if enabled)
2. Receives network messages from P2P layer
3. Runs mining attempts continuously
4. Batches 10 mined addresses and submits on-chain claims

All events are serialized as `GameEvent<(Address, U256), Position>` and gossiped via libp2p.

**Thread-Safe State Management**: The World uses `Arc<RwLock<>>` for shared state:
- `PlayersPool`: HashMap of all connected players
- `mining_rewards`: Counter of successful mines
- `mined`: HashSet of mined addresses pending claim
- `messages`: Chat message history

**Feature Flags**:
- `game_core[interface]`: Enables Bevy UI (default off)
- `game_interface[custom_sprites]`: Enables custom sprite rendering (default on)

### Mining Logic

Mining happens in `crates/contract/src/mine/miner.rs:37`:
```rust
// keccak256(abi.encodePacked(nonce, minerAddress))
let salt = keccak256((nonce, self.address).abi_encode_packed());
let mined = self.factory.create2(salt, init_hash);

// If mined address < difficulty threshold, success
if mined < self.difficulty { ... }
```

The miner increments a nonce counter on each attempt. When 10 successful mines are accumulated, they're batch-submitted to the contract via `processMiningArray()`.

### Network Protocol

P2P communication uses a single gossipsub topic `"game_events"`. All GameEvent types (PlayerMovement, PlayerFound, ChatMessage, Quit) are JSON-serialized and gossiped to peers. The network layer is in `crates/network/src/p2p/`.

### Blockchain Integration

Contract interaction via Alloy:
- RPC endpoint: `https://mainnet.base.org` (chain ID 8453)
- Contract calls: `difficulty()`, `initHash()`, `processMiningArray()`
- Transaction flow: submit batch → register pending tx → wait for confirmation
- Spawns async tasks for claim transactions to avoid blocking the main loop

## Development Configuration

The `game_core` crate uses profile optimizations:
```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3
```

This enables moderate optimization for the core crate while fully optimizing dependencies, balancing compilation speed with runtime performance during development.

## Important Implementation Details

- The main event loop never exits until `ExitStatus.exit()` is called (on Quit event or Ctrl+C).
- Player positions are tracked as `Position` (delta movements, not absolute coordinates).
- When a new player's movement is seen, they're auto-added to the players pool with default position.
- Camera boundaries in the interface prevent camera movement until the player is 150px from center.
- Mining runs continuously in the main loop with no throttling - one attempt per iteration.
