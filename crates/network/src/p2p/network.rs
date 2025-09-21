use crate::prelude::*;
use libp2p::{
    StreamProtocol, Swarm,
    futures::StreamExt,
    gossipsub::{IdentTopic, Message},
    identity::Keypair,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use tokio::sync::mpsc;
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
    T: Into<IdentTopic> + Send + Sync + 'static,
    M: Into<Vec<u8>> + Send + Sync + 'static,
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
            .with_dns()?
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
        loop {
            tokio::select! {
                biased;
                Some((topic, data)) = self.receiver.recv() => {
                        debug!("Received message from receiver");
                        if let Err(e) = self.send(topic, data) {
                            error!("Send error: {e:?}");
                        }
                }
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(
                        list,
                    ))) => {
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
                    SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { message, .. },
                    )) => {
                        info!("Received gossipsub message: {:?}", message);
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

        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic, data) {
            error!("Publish error: {e:?}");
        }

        debug!("Sending Gossipsub message to topic");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::level_filters::LevelFilter;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_build() -> Result<()> {
        tracing_subscriber::fmt::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .init();

        // Network 1
        let topic = IdentTopic::new("test");
        let keypair = Keypair::generate_ed25519();
        let (sender, mut listener) = Peer2Peer::build(keypair)
            .unwrap()
            .start(vec![topic.clone()]);

        loop {
            tokio::select! {
                _ = sleep(Duration::from_secs(5)) => {
                    sender
                        .send((topic.clone(), b"Hello, world!"))
                        .await
                        .unwrap();
            }
                message = listener.recv() => match message {
                    Some(message) => {
                        info!("Received message: {:?}", message);
                    },
                    None => return Err(anyhow!("Listener closed"))
                }

            }
        }
    }
}
