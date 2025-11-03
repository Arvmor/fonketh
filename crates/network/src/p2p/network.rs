use crate::prelude::*;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{IdentTopic, Message};
use libp2p::identity::Keypair;
use libp2p::swarm::dial_opts::DialOpts;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{StreamProtocol, Swarm};
use std::{fmt::Debug, time::Duration};
use tokio::sync::mpsc::{self, error::TryRecvError};

/// Game protocol name
pub const GAME_PROTO_NAME: StreamProtocol = StreamProtocol::new("/game/kad/1.0.0");
/// Listen address for the peer
const LISTEN_ADDR: &str = "/ip4/0.0.0.0/udp/0/quic-v1";
/// Bootstrap nodes for the peer
const BOOTSTRAP_NODES: [&str; 1] = ["/ip4/13.220.20.144/udp/7331/quic-v1"];
/// Topics to subscribe to
const TOPICS: [&str; 1] = ["game_events"];

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kad: kad::Behaviour<kad::store::MemoryStore>,
}

pub trait GossipTypes {
    type Topic;
    type Data;
}

pub trait Network: GossipTypes {
    fn listen(&mut self) -> Result<()>;
    fn subscribe(&mut self, topic: Self::Topic) -> Result<()>;
    fn send(&mut self, topic: Self::Topic, data: Self::Data) -> Result<()>;
}

pub struct Peer2Peer<M> {
    pub sender: mpsc::Sender<M>,
    receiver: mpsc::Receiver<M>,
    pub listener: Option<mpsc::Receiver<Message>>,
    talker: mpsc::Sender<Message>,
    swarm: Swarm<MyBehaviour>,
}

impl<M> Peer2Peer<M>
where
    M: Into<Vec<u8>> + Send + Sync + 'static + Debug,
{
    pub fn build(keypair: Keypair) -> Result<Self> {
        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
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

        let (sender, receiver) = mpsc::channel(100);
        let (talker, listener) = mpsc::channel(100);

        Ok(Self {
            swarm,
            sender,
            receiver,
            listener: Some(listener),
            talker,
        })
    }

    async fn run(mut self) -> Result<()> {
        info!("Running network PeerID: {}", self.swarm.local_peer_id());
        self.listen()?;

        // Connect to the bootstrap nodes
        for node in BOOTSTRAP_NODES {
            let opts = DialOpts::unknown_peer_id().address(node.parse()?).build();
            self.swarm.dial(opts)?;
        }

        for topic in TOPICS {
            self.subscribe(topic)?;
        }

        // Kick it off
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        loop {
            tokio::select! {
                _ = interval.tick() => loop {
                    match self.receiver.try_recv() {
                        Err(TryRecvError::Empty) => break,
                        Ok(data) => {
                            if let Err(e) = self.send(TOPICS[0], data) {
                                error!("Publish error: {e:?}");
                            }
                        },
                        Err(e) => {
                            error!("Receive error: {e:?}");
                            return Err(e.into());
                        },
                    }
                },
                SwarmEvent::Behaviour(event) = self.swarm.select_next_some() => match event {
                    MyBehaviourEvent::Mdns(mdns::Event::Discovered(list)) => {
                        for (peer_id, multiaddr) in list {
                            info!("mDNS discovered a new peer: {peer_id}");
                            let behaviour = self.swarm.behaviour_mut();
                            behaviour.kad.add_address(&peer_id, multiaddr);
                            behaviour.gossipsub.add_explicit_peer(&peer_id);
                            info!("Added explicit peer {peer_id}");
                        }
                    }
                    MyBehaviourEvent::Mdns(mdns::Event::Expired(list)) => {
                        for (peer_id, multiaddr) in list {
                            info!("mDNS discover peer has expired: {peer_id}");
                            let behaviour = self.swarm.behaviour_mut();
                            behaviour.kad.remove_address(&peer_id, &multiaddr);
                            behaviour.gossipsub.remove_explicit_peer(&peer_id);
                            info!("Removed explicit peer {peer_id}");
                        }
                    }
                    MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message,
                        ..
                    }) => {
                        info!("Gossipsub message received: {message:?}",);
                        self.talker.send(message).await.unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn start(mut self) -> (mpsc::Sender<M>, mpsc::Receiver<Message>) {
        let sender = self.sender.clone();
        let listener = self.listener.take().unwrap();
        tokio::spawn(self.run());
        (sender, listener)
    }
}

impl<M> GossipTypes for Peer2Peer<M> {
    type Topic = &'static str;
    type Data = M;
}

impl<M> Network for Peer2Peer<M>
where
    M: Into<Vec<u8>> + Send + Sync + 'static,
{
    fn listen(&mut self) -> Result<()> {
        self.swarm.listen_on(LISTEN_ADDR.parse()?)?;

        Ok(())
    }

    fn subscribe(&mut self, topic: Self::Topic) -> Result<()> {
        let topic = IdentTopic::new(topic);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        Ok(())
    }

    fn send(&mut self, topic: Self::Topic, data: Self::Data) -> Result<()> {
        let topic = IdentTopic::new(topic);
        let data = data.into();

        debug!("Publishing Gossipsub topic {topic:?} with data {data:?}",);
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;

        Ok(())
    }
}
