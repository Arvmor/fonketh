use crate::prelude::*;
use libp2p::{
    StreamProtocol, Swarm,
    futures::StreamExt,
    gossipsub::{IdentTopic, Message},
    identity::Keypair,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::{fmt::Debug, time::Duration};
use tokio::sync::mpsc::{self, error::TryRecvError};
pub const GAME_PROTO_NAME: StreamProtocol = StreamProtocol::new("/game/kad/1.0.0");

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

pub struct Peer2Peer<T, M> {
    pub sender: mpsc::Sender<(T, M)>,
    receiver: mpsc::Receiver<(T, M)>,
    pub listener: Option<mpsc::Receiver<Message>>,
    talker: mpsc::Sender<Message>,
    swarm: Swarm<MyBehaviour>,
}

impl<T, M> Peer2Peer<T, M>
where
    T: Into<IdentTopic> + Send + Sync + 'static + Debug,
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

    async fn run(mut self, topics: Vec<T>) -> Result<()> {
        info!("Running network");
        self.listen()?;
        for topic in topics {
            self.subscribe(topic)?;
        }

        // Kick it off
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        loop {
            tokio::select! {
                _ = interval.tick() => loop {
                    match self.receiver.try_recv() {
                        Err(TryRecvError::Empty) => break,
                        Ok((topic, data)) => {
                            if let Err(e) = self.send(topic, data) {
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

    pub fn start(mut self, topics: Vec<T>) -> (mpsc::Sender<(T, M)>, mpsc::Receiver<Message>) {
        let sender = self.sender.clone();
        let listener = self.listener.take().unwrap();
        tokio::spawn(self.run(topics));
        (sender, listener)
    }
}

impl<T, M> GossipTypes for Peer2Peer<T, M> {
    type Topic = T;
    type Data = M;
}

impl<T, M> Network for Peer2Peer<T, M>
where
    T: Into<IdentTopic> + Send + Sync + 'static,
    M: Into<Vec<u8>> + Send + Sync + 'static,
{
    fn listen(&mut self) -> Result<()> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(())
    }

    fn subscribe(&mut self, topic: Self::Topic) -> Result<()> {
        let topic = topic.into();
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        Ok(())
    }

    fn send(&mut self, topic: Self::Topic, data: Self::Data) -> Result<()> {
        let topic = topic.into();
        let data = data.into();

        debug!("Publishing Gossipsub topic {topic:?} with data {data:?}",);
        self.swarm.behaviour_mut().gossipsub.publish(topic, data)?;

        Ok(())
    }
}
