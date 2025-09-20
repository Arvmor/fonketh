use crate::prelude::*;
use libp2p::{
    StreamProtocol, Swarm,
    futures::StreamExt,
    gossipsub::{IdentTopic, TopicHash},
    identity::Keypair,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub const GAME_PROTO_NAME: StreamProtocol = StreamProtocol::new("/game/kad/1.0.0");

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
}

pub trait Network {
    type Connection;

    fn connect(&self) -> Result<Self::Connection>;
    fn send(&self, topic: impl Into<TopicHash>, data: impl Into<Vec<u8>>) -> Result<()>;
}

pub struct Peer2Peer {
    topic: IdentTopic,
    swarm: Swarm<MyBehaviour>,
}

impl Peer2Peer {
    fn new(swarm: Swarm<MyBehaviour>) -> Self {
        // let swarm = Arc::new(Mutex::new(swarm));
        let topic = IdentTopic::new("game_events");
        Self { swarm, topic }
    }

    pub fn build(keypair: Keypair) -> Result<Self> {
        let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            // Create behavior.
            .with_behaviour(|key| {
                // Set a custom gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default().build()?;

                // build a gossipsub network behaviour
                let gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                )?;

                let mdns = mdns::tokio::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?;

                let cfg = kad::Config::new(GAME_PROTO_NAME);
                let store = kad::store::MemoryStore::new(key.public().to_peer_id());
                let kad = kad::Behaviour::with_config(key.public().to_peer_id(), store, cfg);

                Ok(MyBehaviour {
                    gossipsub,
                    mdns,
                    kad,
                })
            })?
            .build();

        // Listen on all interfaces and whatever port the OS assigns
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        let topic = IdentTopic::new("game_events");
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        info!("Swarm subscribed to topic: {topic}");

        Ok(Self::new(swarm))
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            let event = self.swarm.select_next_some().await;
            match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!("mDNS discovered a new peer: {peer_id}");
                        let behaviour = self.swarm.behaviour_mut();
                        behaviour.kad.add_address(&peer_id, multiaddr);
                        behaviour.gossipsub.add_explicit_peer(&peer_id);
                        info!("Added explicit peer {peer_id}");
                    }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!("mDNS discover peer has expired: {peer_id}");
                        let behaviour = self.swarm.behaviour_mut();
                        behaviour.kad.remove_address(&peer_id, &multiaddr);
                        behaviour.gossipsub.remove_explicit_peer(&peer_id);
                        info!("Removed explicit peer {peer_id}");
                    }
                }
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => info!(
                    "Got message: '{}' with id: {id} from peer: {peer_id}",
                    String::from_utf8_lossy(&message.data),
                ),
                _ => {}
            }

            // Send message to all peers
            self.swarm
                .behaviour_mut()
                .gossipsub
                .publish(self.topic.clone(), b"kossher tar");
        }
    }
}

impl Network for Peer2Peer {
    type Connection = ();

    fn connect(&self) -> Result<Self::Connection> {
        Ok(())
    }

    fn send(&self, topic: impl Into<TopicHash>, data: impl Into<Vec<u8>>) -> Result<()> {
        let data = data.into();
        let topic = topic.into();

        // let mut swarm = self.swarm.lock().await;
        // if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, data) {
        //     error!("Publish error: {e:?}");
        // }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::level_filters::LevelFilter;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_build() {
        tracing_subscriber::fmt::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .init();

        // Network 1
        tokio::spawn(async move {
            let keypair = Keypair::ed25519_from_bytes([1; 32]).unwrap();
            let mut network = Peer2Peer::build(keypair).unwrap();
            network.run().await.unwrap()
        });
        sleep(Duration::from_secs(5)).await;

        tokio::spawn(async move {
            let keypair = Keypair::ed25519_from_bytes([2; 32]).unwrap();
            let mut network = Peer2Peer::build(keypair).unwrap();
            network.run().await.unwrap()
        });
        sleep(Duration::from_secs(10)).await;
    }
}
