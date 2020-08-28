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
use libp2p::kad::{Kademlia, KademliaConfig, KademliaEvent, QueryId};

use libp2p::mdns::{Mdns, MdnsEvent};

use libp2p::core::connection::ConnectionId;
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction};
use libp2p::swarm::{PollParameters, ProtocolsHandler};

use libp2p::core::multiaddr::Protocol;
use libp2p::{Multiaddr, PeerId};

use log::{debug, error, info};

pub const LOCHA_KAD_PROTOCOL_NAME: &[u8] = b"/locha/kad/1.0.0";

/// Builder for [`DiscoveryBehaviour`](struct.DiscoveryBehaviour.html).
#[derive(Debug)]
pub struct DiscoveryBuilder {
    use_mdns: bool,

    allow_ipv4_private: bool,
    allow_ipv6_link_local: bool,
    allow_ipv6_ula: bool,
    id: Option<PeerId>,
}

impl DiscoveryBuilder {
    /// Create a new [`DiscoveryBehaviour`](struct.DiscoveryBehaviour.html)
    /// builder.
    pub fn new() -> DiscoveryBuilder {
        DiscoveryBuilder {
            use_mdns: false,

            allow_ipv4_private: false,
            allow_ipv6_link_local: false,
            allow_ipv6_ula: false,
            id: None,
        }
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

    /// Allow IPv6 link local addresses (fe00::/7)?
    pub fn allow_ipv6_link_local(&mut self, v: bool) -> &mut Self {
        self.allow_ipv6_link_local = v;
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

    /// Build the discovery behaviour.
    pub fn build(&mut self) -> DiscoveryBehaviour {
        let id = self
            .id
            .clone()
            .expect("PeerId is necessary to participate in Peer Discovery");

        let mut kad_config = KademliaConfig::default();
        kad_config.set_protocol_name(LOCHA_KAD_PROTOCOL_NAME);

        DiscoveryBehaviour {
            mdns: if self.use_mdns {
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
            kademlia: Kademlia::with_config(
                id.clone(),
                MemoryStore::new(id),
                kad_config,
            ),
            pending_events: VecDeque::new(),

            allow_ipv4_private: self.allow_ipv4_private,
            allow_ipv6_link_local: self.allow_ipv6_link_local,
            allow_ipv6_ula: self.allow_ipv6_ula,
        }
    }
}

impl Default for DiscoveryBuilder {
    fn default() -> DiscoveryBuilder {
        DiscoveryBuilder::new()
    }
}

/// Discovery behaviour
pub struct DiscoveryBehaviour {
    /// Mdns to locate neighboring peers
    mdns: Option<Mdns>,
    /// Kademlia network behaviour
    kademlia: Kademlia<MemoryStore>,
    pending_events: VecDeque<DiscoveryEvent>,

    allow_ipv4_private: bool,
    allow_ipv6_link_local: bool,
    allow_ipv6_ula: bool,
}

impl DiscoveryBehaviour {
    #[rustfmt::skip]
    fn is_address_allowed(&self, addr: &Multiaddr) -> bool {
        (self.allow_ipv4_private && is_ipv4_private(&addr)) ||
        (self.allow_ipv6_link_local && is_ipv6_link_local(&addr)) ||
        (self.allow_ipv6_ula && is_ipv6_ula(addr))
    }
}

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
            if !self.is_address_allowed(&addr) {
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
                if !self.is_address_allowed(&addr) {
                    debug!(
                        target: "locha-p2p",
                        "mDNS address {} not allowed",
                        addr
                    );
                    continue;
                }

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
                        KademliaEvent::QueryResult { .. } => {
                            debug!(target: "locha-p2p", "query result produced");
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
    let res = addr.iter().next().map(|p| {
        if let Protocol::Ip4(ipv4) = p {
            ipv4.is_private()
        } else {
            false
        }
    });

    match res {
        Some(v) => v,
        None => false,
    }
}

/// Is the multiaddress a link local IPv6?
fn is_ipv6_link_local(addr: &Multiaddr) -> bool {
    let res = addr.iter().next().map(|p| {
        if let Protocol::Ip6(ipv6) = p {
            (ipv6.segments()[0] & 0xffc0) == 0xfe80
        } else {
            false
        }
    });

    match res {
        Some(v) => v,
        None => false,
    }
}

/// Is the multiaddress an IPv6 ULA?
fn is_ipv6_ula(addr: &Multiaddr) -> bool {
    let res = addr.iter().next().map(|p| {
        if let Protocol::Ip6(ipv6) = p {
            (ipv6.segments()[0] & 0xfe00) == 0xfc00
        } else {
            false
        }
    });

    match res {
        Some(v) => v,
        None => false,
    }
}
