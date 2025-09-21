pub mod p2p;

// Crate Public API
pub use p2p::{GAME_PROTO_NAME, Network, Peer2Peer};

// Crate Prelude
pub mod prelude {
    pub use anyhow::{Result, anyhow};
    pub use libp2p::PeerId;
    pub use libp2p::identity::Keypair;
    pub use libp2p::{gossipsub, kad, mdns, noise, tcp, yamux};
    pub use tracing::{debug, error, info, trace, warn};
}
