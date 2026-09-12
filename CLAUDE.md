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

# Run the game with the Bevy window (what releases ship)
cargo run -p game_app -F interface --release

# Run tests
cargo test

# Run tests for specific crate
cargo test -p game_contract
cargo test -p game_network
```

## Releases and Installer

- `.github/workflows/release.yml` runs on `v*` tags (or manually) and builds `game_app` with `-F interface` for macOS (arm64, x86_64), Windows (x86_64) and Linux (x86_64). Each archive `fonketh-<tag>-<target>.tar.gz|.zip` unpacks to `bin/fonketh`, `assets/`, `README.md`, `VERSION`; `SHA256SUMS.txt` covers all of them. Features shipped are set by `RELEASE_FEATURES` in the workflow.
- `install.sh` (macOS/Linux, POSIX sh) and `install.ps1` (Windows) are the end-user wizards: detect platform, resolve the latest tag via the `releases/latest` redirect (no API calls), download + verify + unpack into `~/.fonketh` / `%LOCALAPPDATA%\Programs\Fonketh`, install a launcher that `cd`s into that directory before starting the binary (so `./private.key` and sprite composites land there), optionally edit PATH, and set up the miner key. Both support `--archive` for offline installs and `--uninstall`. Details in `docs/INSTALL.md`.
- Asset lookup lives in `crates/interface/src/assets.rs`: `FONKETH_ASSETS` env → `CARGO_MANIFEST_DIR/../..` (cargo run) → exe dir, its parent or grandparent → cwd, first one containing `assets/`. Bevy paths are given relative to that root (`assets/textures/...`), never relative to cwd.

## Private Key Management

The app loads a private key in the following priority order:
1. `PRIVATE_KEY` environment variable
2. Command-line argument (path to key file)
3. `./private.key` file (default)
4. Generates a new key and saves to `./private.key` if none found

## Architecture

### Workspace Structure

The codebase is organized as a Cargo workspace with 7 crates:

- **game_app** (`crates/app`): Main binary entry point. Initializes the World with a Character and starts the game loop. Features `interface` and `mine` forward to `game_core`.

- **game_core** (`crates/core`): Core game logic containing the World state management, player pool, and main event loop. The World orchestrates three main components:
  - Network layer (P2P gossip)
  - Mining contract client (CREATE2 mining)
  - Interface layer (optional, Bevy UI)

- **game_contract** (`crates/contract`): Blockchain interaction layer using Alloy. Contains:
  - `RewarderClient`: Interfaces with the Base chain contract
  - `Miner`: Implements CREATE2 mining logic, attempting to find addresses below network difficulty
  - Contract ABI loaded from `contracts/rewarder.json`

- **game_network** (`crates/network`): P2P networking using libp2p with gossipsub, mDNS discovery, and Kademlia DHT.

- **game_interface** (`crates/interface`): Optional Bevy-based game UI. Enabled via `interface` feature flag in game_core. Handles keyboard input, camera, menus, HUD, chat, and sprite rendering. Key modules:
  - `screen.rs`: `Screen` state (`Menu` → `Playing` ⇄ `Paused`). Menus are overlays with `DespawnOnExit`; the world renders behind them.
  - `theme.rs`: single source of colors, type scale, spacing and bundle helpers (`panel`, `text`, `caption`, `shorten`).
  - `menu.rs`: start and pause menus; keyboard (arrows/WASD + Enter) and pointer navigation over `MenuButton`s.
  - `hud.rs`: mining stats (session total, claim progress bar, claims), miner identity, online count, chat panel, controls hint.
  - `input.rs`: one `route_keyboard` system dispatches key presses by screen and chat focus, so keys never reach two consumers.
  - `chat.rs`: renders chat rows from the `ChatEntry` trait (sender/content/age) and the input field with caret.
  - `stats.rs` / `toast.rs`: derive session totals and claims from the world's pending-batch counter and show toasts.
  - Preview without RPC or peers: `cargo run -p game_interface --example preview`.

- **game_primitives** (`crates/primitives`): Shared types and traits (GameEvent, WorldState, ExitStatus, Identifier, ChatEntry).

- **game_sprite** (`crates/sprite`): Character sprite asset handling (optional dependency of game_interface). Picks a `Character` variant, a `Hat` and a hair color from the keccak256 hash of the peer ID, recolors the hair, composites the hat overlay, and writes `mod_{character}-{id}.png` (gitignored) next to the sheets. Sheets live in `assets/textures/characters/` (all 168x24, seven 24x24 frames) and hat overlays in `assets/textures/hats/`.

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

- The main event loop never exits until `ExitStatus.exit()` is called (on Quit event or Ctrl+C). In the UI, Quit is only reachable from the start/pause menu; `Esc` pauses instead of quitting.
- Player positions are tracked as `Position` (delta movements, not absolute coordinates).
- When a new player's movement is seen, they're auto-added to the players pool with default position.
- Camera boundaries in the interface prevent camera movement until the player is 150px from center.
- Mining runs continuously in the main loop with no throttling - one attempt per iteration.
