# Fonketh - P2P Mining Pool Game

## Project Overview

Fonketh is a **Peer-to-Peer Mining Pool** implemented as a **Gamified Proof-of-Work Node**. It combines blockchain mining mechanics with a multiplayer game interface, allowing players to mine CREATE2 addresses and earn ERC-20 tokens based on their mining contributions.

### Key Features

- **Decentralized Mining Pool**: Players join a P2P network to collectively mine CREATE2 addresses
- **Gamified Interface**: 2D character-based game with movement, chat, and visual feedback
- **Blockchain Integration**: Live contract on Base Chain for reward distribution
- **Real-time Multiplayer**: P2P networking with gossip protocol for player coordination
- **Token Rewards**: ERC-20 tokens distributed based on mining success

## Development Workflow

### Code Style and Standards

1. **Formatting**: Always use nightly rustfmt

   ```bash
   cargo +nightly fmt --all
   ```

2. **Linting**: Run clippy with all features
   ```bash
   RUSTFLAGS="-D warnings" cargo +nightly clippy --workspace --lib --examples --tests --benches --all-features --locked
   ```

## Architecture

### Project Structure

```
fonketh/
├── crates/                    # Rust workspace crates
│   ├── app/                   # Main application entry point
│   ├── contract/              # Blockchain contract integration
│   ├── core/                  # Core game logic and world management
│   ├── interface/             # 2D game interface and rendering
│   ├── network/               # P2P networking implementation
│   ├── primitives/            # Common data types and events
│   └── sprite/                # Sprite rendering and color management
├── contracts/                 # Smart contract files
│   ├── Rewarder.sol          # Main reward distribution contract
│   └── rewarder.json         # Contract ABI
├── assets/                    # Game assets
│   └── textures/             # Character sprites and backgrounds
├── docs/                      # Documentation and diagrams
└── Dockerfile                # Container configuration
```

### Core Components

#### 1. **App Crate** (`crates/app/`)

- **Purpose**: Main application entry point
- **Key Files**: `main.rs`
- **Functionality**:
  - Initializes the game world
  - Loads private key for blockchain interaction
  - Sets up logging and tracing
  - Starts the core game loop

#### 2. **Core Crate** (`crates/core/`)

- **Purpose**: Central game logic and world management
- **Key Modules**:
  - `world.rs`: Main game world with player management
  - `player/`: Character system and movement
  - `map/`: World state and coordinate system
  - `movements/`: Position and movement logic
- **Functionality**:
  - Manages player pool and game state
  - Handles mining rewards and batch processing
  - Coordinates between network and interface
  - Processes game events (movement, mining, chat)

#### 3. **Contract Crate** (`crates/contract/`)

- **Purpose**: Blockchain integration and mining logic
- **Key Files**:
  - `lib.rs`: Rewarder client for contract interaction
  - `mine/miner.rs`: CREATE2 mining implementation
- **Functionality**:
  - Connects to Base Chain via RPC
  - Implements CREATE2 address mining algorithm
  - Manages reward distribution transactions
  - Handles contract state synchronization

#### 4. **Network Crate** (`crates/network/`)

- **Purpose**: P2P networking and peer discovery
- **Key Files**: `p2p/network.rs`
- **Functionality**:
  - Implements libp2p-based networking
  - Uses GossipSub for message broadcasting
  - Includes mDNS for local peer discovery
  - Supports Kademlia DHT for peer routing
  - Handles QUIC and TCP transport protocols

#### 5. **Interface Crate** (`crates/interface/`)

- **Purpose**: 2D game interface and user interaction
- **Key Modules**:
  - `logic/`: Game loop and event handling
  - `components/`: UI components and rendering
  - `movements/`: Keyboard input handling
  - `chat/`: Chat system implementation
- **Functionality**:
  - Renders 2D character sprites
  - Handles keyboard input for movement
  - Manages chat interface
  - Displays mining progress and rewards

#### 6. **Primitives Crate** (`crates/primitives/`)

- **Purpose**: Common data types and game events
- **Key Files**: `events.rs`
- **Functionality**:
  - Defines game event types (movement, mining, chat, quit)
  - Provides serialization/deserialization
  - Manages game state transitions

#### 7. **Sprite Crate** (`crates/sprite/`)

- **Purpose**: Sprite rendering and color management
- **Key Files**: `lib.rs`
- **Functionality**:
  - Loads and processes sprite images
  - Generates unique colors based on player identifiers
  - Modifies sprite colors for player differentiation
  - Handles sprite sheet processing

## Networking

### P2P Architecture

- **Protocol**: libp2p with GossipSub
- **Transport**: QUIC and TCP
- **Discovery**: mDNS for local peers, Kademlia DHT for global discovery

## Development

### Prerequisites

- Rust 1.70+
- Docker (optional)
- RPC access to the blockchain

### Building

```bash
# Build the project
cargo build --release

# Run with private key
cargo run --release --bin app <path_to_private_key>

# Or set environment variable
PRIVATE_KEY=<your_private_key> cargo run --release --bin app
```

## Contributing

This project uses a modular Rust workspace architecture. Each crate has a specific responsibility and can be developed independently. The main integration points are:

1. **Core ↔ Network**: Game events and player state synchronization
2. **Core ↔ Contract**: Mining results and reward distribution
3. **Core ↔ Interface**: User input and visual feedback
4. **Network ↔ All**: P2P message broadcasting and reception
