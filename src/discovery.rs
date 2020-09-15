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

//! # Discovery behaviour

use std::collections::VecDeque;
use std::task::{Context, Poll};

use libp2p::kad::handler::{KademliaHandler, KademliaHandlerEvent};
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::QueryResult;
use libp2p::kad::{Kademlia, KademliaConfig, KademliaEvent, QueryId};

use libp2p::mdns::{Mdns, MdnsEvent};

use libp2p::core::connection::ConnectionId;
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction};
use libp2p::swarm::{PollParameters, ProtocolsHandler};

use libp2p::core::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};

use log::{debug, error, info};

pub const LOCHA_KAD_PROTOCOL_NAME: &[u8] = b"/locha/kad/1.0.0";
pub const BOOTSTRAP_NODES: &[&str] = &["/dns/p2p.locha.io/tcp/45215/p2p/16Uiu2HAm3U4JmNLwVfCypZX3hCLmVkcsdzEh8NHfPFcKRhsaJ8rf"];

/// Configuration builder for [`DiscoveryBehaviour`].
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    use_mdns: bool,

    allow_ipv4_private: bool,
    allow_ipv4_shared: bool,
    allow_ipv6_ula: bool,
    id: Option<PeerId>,

    bootstrap: Vec<(PeerId, Multiaddr)>,
}

impl DiscoveryConfig {
    /// Create a new [`DiscoveryConfig`]
    pub fn new(use_default_bootstrap_nodes: bool) -> DiscoveryConfig {
        let mut cfg = DiscoveryConfig {
            use_mdns: false,

            allow_ipv4_private: false,
            allow_ipv4_shared: false,
            allow_ipv6_ula: false,
            id: None,

            bootstrap: Vec::new(),
        };

        if use_default_bootstrap_nodes {
            for addr_str in BOOTSTRAP_NODES.iter() {
                let mut addr = addr_str
                    .parse::<Multiaddr>()
                    .expect("Invalid bootstrap node");

                match addr.pop() {
                    Some(Protocol::P2p(hash)) => {
                        let peer_id = PeerId::from_multihash(hash).unwrap();
                        cfg.add_address(&peer_id, &addr);
                    }
                    _ => panic!("Bootstrap node doesn't have a PeerId"),
                }
            }
        }

        cfg
    }

    /// Use mDNs for peer discovery?
    pub fn use_mdns(&mut self, v: bool) -> &mut Self {
        self.use_mdns = v;
        self
    }

    /// Allow IPv4 private addresses?
    ///
    /// Address ranges considered private:
    ///
    /// - 10.0.0.0/8
    /// - 172.16.0.0/12
    /// - 192.168.0.0/16
    pub fn allow_ipv4_private(&mut self, v: bool) -> &mut Self {
        self.allow_ipv4_private = v;
        self
    }

    /// Allow IPv4 shared addresses?
    ///
    /// Addresses considered part of Shared Address Space:
    ///
    /// - 100.64.0.0/10
    pub fn allow_ipv4_shared(&mut self, v: bool) -> &mut Self {
        self.allow_ipv4_shared = v;
        self
    }

    /// Allow unique local addresses (fc00::/7)?
    pub fn allow_ipv6_ula(&mut self, v: bool) -> &mut Self {
        self.allow_ipv6_ula = v;
        self
    }

    /// Peer Id of the node discovering peers.
    pub fn id(&mut self, id: PeerId) -> &mut Self {
        self.id = Some(id);
        self
    }

    /// Add a bootstrap address for Kademlia DHT
    pub fn add_address(
        &mut self,
        peer_id: &PeerId,
        addr: &Multiaddr,
    ) -> &mut Self {
        self.bootstrap.push((peer_id.clone(), addr.clone()));
        self
    }
}

impl Default for DiscoveryConfig {
    fn default() -> DiscoveryConfig {
        DiscoveryConfig::new(false)
    }
}

/// Discovery behaviour
pub struct DiscoveryBehaviour {
    /// Mdns to locate neighboring peers
    mdns: Option<Mdns>,
    /// Kademlia network behaviour
    kademlia: Kademlia<MemoryStore>,
    pending_events: VecDeque<DiscoveryEvent>,

    config: DiscoveryConfig,
}

impl DiscoveryBehaviour {
    pub fn with_config(mut config: DiscoveryConfig) -> DiscoveryBehaviour {
        let id = config
            .id
            .clone()
            .expect("PeerId is necessary to participate in Peer Discovery");

        let mut kad_config = KademliaConfig::default();
        kad_config.set_protocol_name(LOCHA_KAD_PROTOCOL_NAME);

        let mut kademlia =
            Kademlia::with_config(id.clone(), MemoryStore::new(id), kad_config);

        let bootstrap_nodes =
            std::mem::replace(&mut config.bootstrap, Vec::new());

        for (peer, addr) in bootstrap_nodes {
            kademlia.add_address(&peer, addr);
        }

        DiscoveryBehaviour {
            mdns: if config.use_mdns {
                match Mdns::new() {
                    Ok(m) => {
                        info!(target: "locha-p2p", "using mDNS for peer discovery");
                        Some(m)
                    }
                    Err(e) => {
                        error!(target: "locha-p2p", "failed to initialize mDNS: {}", e);
                        None
                    }
                }
            } else {
                None
            },
            kademlia,
            pending_events: VecDeque::new(),
            config,
        }
    }

    /// Start bootstrap process
    pub fn bootstrap(&mut self) {
        if let Err(e) = self.kademlia.bootstrap() {
            error!(target: "locha-p2p", "Couldn't bootstrap: {}", e);
        }
    }
}

impl DiscoveryBehaviour {
    fn is_address_not_allowed(&self, addr: &Multiaddr) -> bool {
        (!self.config.allow_ipv4_private && is_ipv4_private(&addr))
            || (!self.config.allow_ipv4_shared && is_ipv4_shared(&addr))
            || (!self.config.allow_ipv6_ula && is_ipv6_ula(addr))
    }
}

#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    Discovered(PeerId),
    UnroutablePeer(PeerId),
}

impl NetworkBehaviour for DiscoveryBehaviour {
    type ProtocolsHandler = KademliaHandler<QueryId>;
    type OutEvent = DiscoveryEvent;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        NetworkBehaviour::new_handler(&mut self.kademlia)
    }

    fn addresses_of_peer(&mut self, id: &PeerId) -> Vec<Multiaddr> {
        let mut ret = Vec::new();
        for addr in self.kademlia.addresses_of_peer(id) {
            if self.is_address_not_allowed(&addr) {
                debug!(
                    target: "locha-p2p",
                    "Kad address {} not allowed",
                    addr
                );
                continue;
            }

            ret.push(addr);
        }

        if let Some(ref mut mdns) = self.mdns {
            for addr in mdns.addresses_of_peer(id) {
                ret.push(addr);
            }
        }

        ret
    }

    fn inject_disconnected(&mut self, id: &PeerId) {
        self.kademlia.inject_disconnected(id)
    }

    fn inject_connected(&mut self, id: &PeerId) {
        self.kademlia.inject_connected(id)
    }

    fn inject_event(
        &mut self,
        source: PeerId,
        connection: ConnectionId,
        event: KademliaHandlerEvent<QueryId>,
    ) {
        self.kademlia.inject_event(source, connection, event)
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        parameters: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        // Process pending events first
        if let Some(ev) = self.pending_events.pop_front() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev));
        }

        // Process Kademlia, they might get us some good data than other methods
        // as it's the state of the network.
        while let Poll::Ready(action) = self.kademlia.poll(cx, parameters) {
            match action {
                NetworkBehaviourAction::GenerateEvent(ev) => {
                    let result = match ev {
                        KademliaEvent::QueryResult { result, .. } => {
                            debug!(target: "locha-p2p", "query result produced");
                            match result {
                                QueryResult::Bootstrap(Ok(info)) => info!(
                                    target: "locha-p2p",
                                    "Succesfully bootstrapped with {}",
                                    info.peer
                                ),
                                QueryResult::Bootstrap(Err(e)) => {
                                    error!(
                                        target: "locha-p2p",
                                        "Bootstrap failed: {:?}",
                                        e
                                    );
                                }
                                _ => (),
                            };

                            Poll::Pending
                        }
                        KademliaEvent::RoutingUpdated { peer, .. } => {
                            debug!(target: "locha-p2p", "routing updated {}", peer);
                            let event = DiscoveryEvent::Discovered(peer);
                            Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                                event,
                            ))
                        }
                        KademliaEvent::UnroutablePeer { peer } => {
                            debug!(target: "locha-p2p", "unroutable peer {}", peer);
                            let event = DiscoveryEvent::UnroutablePeer(peer);
                            Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                                event,
                            ))
                        }
                        KademliaEvent::RoutablePeer { peer, .. } => {
                            debug!(target: "locha-p2p", "routable peer {}", peer);
                            let event = DiscoveryEvent::Discovered(peer);
                            Poll::Ready(NetworkBehaviourAction::GenerateEvent(
                                event,
                            ))
                        }
                        KademliaEvent::PendingRoutablePeer { peer, .. } => {
                            debug!(target: "locha-p2p", "pending routable peer {}", peer);
                            Poll::Pending
                        }
                    };

                    // Return only if we have an event.
                    if let Poll::Ready(_) = result {
                        return result;
                    }
                }
                NetworkBehaviourAction::DialAddress { address } => {
                    return Poll::Ready(NetworkBehaviourAction::DialAddress {
                        address,
                    });
                }
                NetworkBehaviourAction::DialPeer { peer_id, condition } => {
                    return Poll::Ready(NetworkBehaviourAction::DialPeer {
                        peer_id,
                        condition,
                    });
                }
                NetworkBehaviourAction::NotifyHandler {
                    peer_id,
                    handler,
                    event,
                } => {
                    return Poll::Ready(
                        NetworkBehaviourAction::NotifyHandler {
                            peer_id,
                            handler,
                            event,
                        },
                    );
                }
                NetworkBehaviourAction::ReportObservedAddr { address } => {
                    return Poll::Ready(
                        NetworkBehaviourAction::ReportObservedAddr { address },
                    );
                }
            }
        }

        // Poll mDNS to see if we found out nodes on our local network.
        if let Some(ref mut mdns) = self.mdns {
            while let Poll::Ready(action) = mdns.poll(cx, parameters) {
                match action {
                    NetworkBehaviourAction::GenerateEvent(event) => match event
                    {
                        MdnsEvent::Discovered(addrs) => {
                            for (peer, addr) in addrs {
                                debug!(target: "locha-p2p", "mDNS discovered peer {} on {}", peer, addr);
                                self.pending_events.push_back(
                                    DiscoveryEvent::Discovered(peer),
                                );
                            }

                            if let Some(ev) = self.pending_events.pop_front() {
                                return Poll::Ready(
                                    NetworkBehaviourAction::GenerateEvent(ev),
                                );
                            }
                        }
                        MdnsEvent::Expired(_) => {
                            debug!(target: "locha-p2p", "mDNS expired some peers");
                        }
                    },
                    NetworkBehaviourAction::DialAddress { address } => {
                        return Poll::Ready(
                            NetworkBehaviourAction::DialAddress { address },
                        );
                    }
                    NetworkBehaviourAction::DialPeer { peer_id, condition } => {
                        return Poll::Ready(NetworkBehaviourAction::DialPeer {
                            peer_id,
                            condition,
                        });
                    }
                    NetworkBehaviourAction::NotifyHandler { event, .. } => {
                        // has no variants
                        match event {}
                    }
                    NetworkBehaviourAction::ReportObservedAddr { address } => {
                        return Poll::Ready(
                            NetworkBehaviourAction::ReportObservedAddr {
                                address,
                            },
                        );
                    }
                }
            }
        }

        Poll::Pending
    }
}

/// Is the multiaddress an IPv4 private address?
fn is_ipv4_private(addr: &Multiaddr) -> bool {
    addr.iter()
        .next()
        .map(|p| {
            if let Protocol::Ip4(ipv4) = p {
                ipv4.is_private()
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Is the multiaddress par of IPv4 Shared Address Space?
fn is_ipv4_shared(addr: &Multiaddr) -> bool {
    addr.iter()
        .next()
        .map(|p| {
            if let Protocol::Ip4(ipv4) = p {
                ipv4.octets()[0] == 100
                    && (ipv4.octets()[1] & 0b1100_0000 == 0b0100_0000)
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Is the multiaddress an IPv6 ULA?
fn is_ipv6_ula(addr: &Multiaddr) -> bool {
    addr.iter()
        .next()
        .map(|p| {
            if let Protocol::Ip6(ipv6) = p {
                (ipv6.segments()[0] & 0xfe00) == 0xfc00
            } else {
                false
            }
        })
        .unwrap_or(false)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::identity::Identity;

    #[test]
    fn test_discovery_config_bootstrap_nodes() {
        // We expect here for parsing of bootstrap nodes to go very well
        DiscoveryConfig::new(true);
    }

    #[test]
    fn test_is_address_not_allowed() {
        let mut config = DiscoveryConfig::new(false);
        config.id(Identity::generate().id());
        let discovery = DiscoveryBehaviour::with_config(config);

        assert!(discovery
            .is_address_not_allowed(&"/ip4/192.168.0.1".parse().unwrap()));
        assert!(discovery
            .is_address_not_allowed(&"/ip4/172.16.0.1".parse().unwrap()));
        assert!(
            discovery.is_address_not_allowed(&"/ip4/10.0.0.1".parse().unwrap())
        );
        assert!(discovery
            .is_address_not_allowed(&"/ip4/100.80.72.1".parse().unwrap()));
        assert!(
            discovery.is_address_not_allowed(&"/ip6/fc00::1".parse().unwrap())
        );
        assert!(!discovery
            .is_address_not_allowed(&"/ip4/186.200.4.1".parse().unwrap()));
        assert!(
            !discovery.is_address_not_allowed(&"/ip6/2001::2".parse().unwrap())
        );
    }

    #[test]
    fn test_is_ipv4_prviate() {
        assert!(is_ipv4_private(&"/ip4/192.168.0.1".parse().unwrap()));
        assert!(is_ipv4_private(&"/ip4/172.16.0.1".parse().unwrap()));
        assert!(is_ipv4_private(&"/ip4/10.0.0.1".parse().unwrap()));
        assert!(!is_ipv4_private(&"/ip4/186.200.4.1".parse().unwrap()));
        assert!(!is_ipv4_private(&"/ip4/100.62.64.1".parse().unwrap()));
        assert!(!is_ipv4_private(&"/dns/p2p.locha.io".parse().unwrap()));
    }

    #[test]
    fn test_is_ipv4_shared() {
        assert!(is_ipv4_shared(&"/ip4/100.80.72.1".parse().unwrap()));
        assert!(!is_ipv4_shared(&"/ip4/186.200.4.1".parse().unwrap()));
        assert!(!is_ipv4_shared(&"/dns/p2p.locha.io".parse().unwrap()));
    }

    #[test]
    fn test_ipv6_is_ula() {
        assert!(is_ipv6_ula(&"/ip6/fc00::1".parse().unwrap()));
        assert!(is_ipv6_ula(&"/ip6/fc20::1".parse().unwrap()));
        assert!(is_ipv6_ula(&"/ip6/fd00::1".parse().unwrap()));
        assert!(!is_ipv6_ula(&"/ip6/2001::1".parse().unwrap()));
        assert!(!is_ipv6_ula(&"/dns/p2p.locha.io".parse().unwrap()));
    }
}
