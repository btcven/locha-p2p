// Copyright 2020 Bitcoin Venezuela and Locha Mesh Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Locha libp2p behaviour

use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::Future;

use libp2p::identify::{Identify, IdentifyEvent};

use libp2p::kad::handler::KademliaHandlerIn;
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::QueryId;
use libp2p::kad::{Kademlia, KademliaConfig, KademliaEvent, QueryResult};

use libp2p::gossipsub::GossipsubRpc;

use libp2p::core::either::EitherOutput;
use libp2p::swarm::toggle::Toggle;
use libp2p::swarm::{DialPeerCondition, PollParameters};
use libp2p::swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess};
use libp2p::NetworkBehaviour;

// Not available on WASM
#[cfg(not(target_os = "unknown"))]
use crate::upnp::UpnpBehaviour;
#[cfg(not(target_os = "unknown"))]
use libp2p::mdns::{Mdns, MdnsEvent};

use wasm_timer::Delay;

use crate::identity::Identity;
use crate::p2p::pubsub::MessageId;
use crate::p2p::pubsub::{Gossipsub, GossipsubEvent, PublishError, Topic};
use crate::{Multiaddr, PeerId, Protocol};

use void::Void;

/// Node/agent version string.
pub const AGENT_VERSION: &str = "Locha Mesh P2P 1.0.0";
/// Locha P2P Kademlia protocol string
pub const LOCHA_KAD_PROTOCOL_NAME: &[u8] = b"/locha/kad/1.0.0";
/// Default bootstrap nodes
pub const BOOTSTRAP_NODES: &[&str] = &["/dns/p2p.locha.io/tcp/45215/p2p/16Uiu2HAm3U4JmNLwVfCypZX3hCLmVkcsdzEh8NHfPFcKRhsaJ8rf"];

/// Bootstrap nodes
pub fn bootstrap_nodes() -> Vec<(PeerId, Multiaddr)> {
    BOOTSTRAP_NODES
        .iter()
        .map(|v| {
            let mut addr = v.parse::<Multiaddr>().unwrap();

            let id = match addr.pop() {
                Some(Protocol::P2p(hash)) => {
                    PeerId::from_multihash(hash).unwrap()
                }
                _ => panic!(),
            };

            (id, addr)
        })
        .collect()
}

#[test]
fn test_parsing_default_bootstraps_nodes_is_ok() {
    assert!(bootstrap_nodes().len() == BOOTSTRAP_NODES.len());
}

/// Stream of BehaviourEvents
pub type BehaviourEventStream = Receiver<BehaviourEvent>;

/// A behaviour event
pub enum BehaviourEvent {
    /// A new message received from a peer
    Message(PeerId, MessageId, Vec<u8>),
}

type InEvent = EitherOutput<
    EitherOutput<
        EitherOutput<
            EitherOutput<Void, KademliaHandlerIn<QueryId>>,
            GossipsubRpc,
        >,
        Void,
    >,
    (),
>;

struct Inner {
    event_chan: Sender<BehaviourEvent>,
    pending_actions: VecDeque<NetworkBehaviourAction<InEvent, ()>>,
    next_query: Delay,
    next_query_time: Duration,
}

impl Inner {
    pub fn new(event_chan: Sender<BehaviourEvent>) -> Inner {
        Inner {
            event_chan,
            pending_actions: VecDeque::new(),
            next_query: Delay::new(Duration::from_secs(0)),
            next_query_time: Duration::from_secs(1),
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(poll_method = "poll")]
pub struct Behaviour {
    #[cfg(not(target_os = "unknown"))]
    mdns: Toggle<Mdns>,
    kademlia: Kademlia<MemoryStore>,
    pubsub: Gossipsub,
    #[cfg(not(target_os = "unknown"))]
    upnp: Toggle<UpnpBehaviour>,
    identify: Identify,

    #[behaviour(ignore)]
    inner: Inner,
}

impl Behaviour {
    pub fn new<'iter>(
        identity: &Identity,
        upnp: bool,
        mdns: bool,
        events_chan_size: usize,
        bootstrap: impl Iterator<Item = &'iter (PeerId, Multiaddr)>,
    ) -> (Behaviour, BehaviourEventStream) {
        let (tx, rx) = channel::<BehaviourEvent>(events_chan_size);

        let mut kad_config = KademliaConfig::default();
        kad_config.set_protocol_name(LOCHA_KAD_PROTOCOL_NAME);
        kad_config.set_query_timeout(Duration::from_secs(300));
        kad_config.disjoint_query_paths(true);

        let id = identity.id();
        let mut behaviour = Behaviour {
            kademlia: Kademlia::with_config(
                id.clone(),
                MemoryStore::new(id),
                kad_config,
            ),
            pubsub: crate::p2p::pubsub::new(identity),
            #[cfg(not(target_os = "unknown"))]
            upnp: Toggle::from(if upnp {
                Some(UpnpBehaviour::new(AGENT_VERSION.into()))
            } else {
                None
            }),
            identify: Identify::new(
                "/locha/identify/1.0.0".into(),
                AGENT_VERSION.into(),
                identity.keypair().public(),
            ),
            #[cfg(not(target_os = "unknown"))]
            mdns: if mdns {
                Self::new_mdns()
            } else {
                Toggle::from(None)
            },

            inner: Inner::new(tx),
        };

        let mut len = 0;
        for (peer, addr) in bootstrap {
            len += 1;
            behaviour.add_peer(&peer, &addr);
        }

        if len != 0 {
            behaviour.bootstrap();
        }

        (behaviour, rx)
    }

    #[cfg(not(target_os = "unknown"))]
    fn new_mdns() -> Toggle<Mdns> {
        let mdns = Mdns::new().ok();
        if mdns.is_some() {
            log::info!(
                target: "locha-p2p",
                "using mDNS for local peer discovery"
            );
        } else {
            log::error!(
                target: "locha-p2p",
                "failed to initialize mDNS"
            );
        }

        Toggle::from(mdns)
    }

    /// Add a peer to the network behaviour.
    ///
    /// - This will add the peer to the Kademlia routing table.
    pub fn add_peer(&mut self, peer: &PeerId, addr: &Multiaddr) {
        log::debug!(target: "locha-p2p", "adding peer {} to routing table", peer);

        self.kademlia.add_address(peer, addr.clone());
    }

    pub fn subscribe(&mut self, topic: Topic) -> bool {
        self.pubsub.subscribe(topic)
    }

    pub fn publish(
        &mut self,
        topic: &Topic,
        data: impl Into<Vec<u8>>,
    ) -> Result<(), PublishError> {
        self.pubsub.publish(topic, data)
    }

    /// Start bootstrap process
    pub fn bootstrap(&mut self) {
        log::info!(target: "locha-p2p", "starting bootstrap process");

        if let Err(e) = self.kademlia.bootstrap() {
            log::error!(target: "locha-p2p", "bootstrap failed: {:?}", e);
        }
    }

    pub fn kademlia(&mut self) -> &mut Kademlia<MemoryStore> {
        &mut self.kademlia
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<InEvent, ()>> {
        // Process pending actions first
        if let Some(act) = self.inner.pending_actions.pop_front() {
            return Poll::Ready(act);
        }

        if let Poll::Ready(_) = Pin::new(&mut self.inner.next_query).poll(cx) {
            let id = PeerId::random();
            log::debug!(
                target: "locha-p2p",
                "starting random Kademlia query with peer {}",
                id
            );
            self.kademlia.get_closest_peers(id);

            self.inner.next_query = Delay::new(self.inner.next_query_time);
            self.inner.next_query_time = std::cmp::min(
                self.inner.next_query_time * 2,
                Duration::from_secs(60),
            );
        }

        Poll::Pending
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for Behaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        if let MdnsEvent::Discovered(addrs) = event {
            for (ref peer, ref addr) in addrs {
                log::debug!(target: "locha-p2p", "mDNS local peer {} on {}", peer, addr);
                // Add peer to Kademlia routing table
                self.add_peer(peer, addr);

                self.inner.pending_actions.push_back(
                    NetworkBehaviourAction::DialPeer {
                        peer_id: peer.clone(),
                        condition: DialPeerCondition::Disconnected,
                    },
                );
            }
        }
    }
}

impl NetworkBehaviourEventProcess<KademliaEvent> for Behaviour {
    fn inject_event(&mut self, event: KademliaEvent) {
        match event {
            KademliaEvent::QueryResult { result, .. } => match result {
                QueryResult::Bootstrap(Ok(info)) => {
                    log::info!(
                        target: "locha-p2p",
                        "bootstrapped succesfully with peer: {}. {} remaining",
                        info.peer, info.num_remaining
                    );
                }
                QueryResult::Bootstrap(Err(_)) => {
                    log::error!(
                        target: "locha-p2p",
                        "bootstrap failed",
                    );
                }
                QueryResult::GetClosestPeers(Ok(info)) => {
                    log::info!(
                        target: "locha-p2p",
                        "found {} peers",
                        info.peers.len(),
                    );
                }
                QueryResult::GetClosestPeers(Err(_)) => {
                    log::error!(
                        target: "locha-p2p",
                        "failed to find closest peers",
                    );
                }
                _ => (),
            },
            KademliaEvent::RoutingUpdated { peer, .. } => {
                self.inner.pending_actions.push_back(
                    NetworkBehaviourAction::DialPeer {
                        peer_id: peer,
                        condition: DialPeerCondition::Disconnected,
                    },
                );
            }
            _ => (),
        }
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for Behaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        if let GossipsubEvent::Message(ref peer, ref id, ref bytes) = event {
            let msg = bytes.data.as_slice().to_vec();

            if let Err(e) = self.inner.event_chan.try_send(
                BehaviourEvent::Message(peer.clone(), id.clone(), msg),
            ) {
                if e.is_full() {
                    log::error!(
                        target: "locha-p2p",
                        "couldn't dispatch behaviour event. Event stream is full"
                    );
                } else if e.is_disconnected() {
                    log::error!(
                        target: "locha-p2p",
                        "couldn't dispatch behaviour event. Event stream is disconnected"
                    );
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<Void> for Behaviour {
    fn inject_event(&mut self, event: Void) {
        match event {}
    }
}

impl NetworkBehaviourEventProcess<IdentifyEvent> for Behaviour {
    fn inject_event(&mut self, event: IdentifyEvent) {
        log::trace!(target: "locha-p2p", "identify: {:?}", event);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::iter;

    #[test]
    fn test_new_behaviour() {
        // Test all parameter combinations

        let (bh1, _) =
            Behaviour::new(&Identity::generate(), true, true, 1, iter::empty());
        #[cfg(not(target_os = "unknown"))]
        assert!(bh1.mdns.is_enabled());
        assert!(bh1.upnp.is_enabled());
        let (bh2, _) = Behaviour::new(
            &Identity::generate(),
            false,
            true,
            1,
            iter::empty(),
        );
        #[cfg(not(target_os = "unknown"))]
        assert!(bh2.mdns.is_enabled());
        assert!(!bh2.upnp.is_enabled());
        let (bh3, _) = Behaviour::new(
            &Identity::generate(),
            true,
            false,
            1,
            iter::empty(),
        );
        #[cfg(not(target_os = "unknown"))]
        assert!(!bh3.mdns.is_enabled());
        assert!(bh3.upnp.is_enabled());
        let (bh4, _) = Behaviour::new(
            &Identity::generate(),
            false,
            false,
            1,
            iter::empty(),
        );
        #[cfg(not(target_os = "unknown"))]
        assert!(!bh4.mdns.is_enabled());
        assert!(!bh4.upnp.is_enabled());

        let bootstrap_nodes = bootstrap_nodes();
        Behaviour::new(
            &Identity::generate(),
            false,
            false,
            1,
            bootstrap_nodes.iter(),
        );
    }

    #[test]
    #[cfg(not(target_os = "unknown"))]
    fn test_new_mdns() {
        let mdns = Behaviour::new_mdns();
        assert!(mdns.is_enabled());
    }
}
