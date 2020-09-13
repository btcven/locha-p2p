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

//! # UPnP
//!
//! This is the Universal Plug and Play (UPnP) `NetworkBehaviour`, this
//! behaviour reports our external IPv4 address if we are behind a NATed
//! router or a general IGD. It also maps all of the open TCP ports for the
//! IPv4 listening addresses found.
//!
//! Each 20 minutes the address is checked and if it differs it's reported
//! again. Also the ports get remapped in case a buggy UPnP implementation
//! drops the mapping, other application messes with it, etc.
//!
//! # Usage
//!
//! ```rust
//! use locha_p2p::upnp::UpnpBehaviour;
//!
//! const PROTOCOL_DESCRIPTION: &str = "My Great Protocol";
//!
//! let _upnp = UpnpBehaviour::new(PROTOCOL_DESCRIPTION.into());
//! ```

use std::borrow::Cow;
use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;
use std::net::Ipv4Addr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Future;

use libp2p::core::connection::ConnectionId;
use libp2p::core::multiaddr::Protocol;
use libp2p::swarm::protocols_handler::DummyProtocolsHandler;
use libp2p::swarm::PollParameters;
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction};
use libp2p::{Multiaddr, PeerId};

use wasm_timer::Delay;

use void::Void;

use log::{error, info, trace};

pub struct UpnpBehaviour {
    interval: Option<Delay>,
    observed_addr: Option<Ipv4Addr>,
    pending_actions: VecDeque<NetworkBehaviourAction<Void, Void>>,
    protocol_desc: Cow<'static, str>,
    ports: HashSet<u16>,
}

impl UpnpBehaviour {
    /// Interval used for checking External IPv4 address and Port Mapping.
    pub const INTERVAL: u64 = 1200;

    /// Create a new [`UpnpBehaviour`]
    ///
    /// # Arguments
    ///
    /// - `protocol_desc`: Protocol description for the port mapping
    /// for UPnP, it's usually displayed on the _User Interface_ of
    /// some routers alongside the port map.
    ///
    /// # Example
    ///
    /// ```rust
    /// use locha_p2p::upnp::UpnpBehaviour;
    ///
    /// let _behaviour = UpnpBehaviour::new("My Great Protocol 1.0.0".into());
    /// ```
    pub fn new(protocol_desc: Cow<'static, str>) -> UpnpBehaviour {
        UpnpBehaviour {
            interval: None,
            observed_addr: None,
            pending_actions: VecDeque::new(),
            protocol_desc,
            ports: HashSet::new(),
        }
    }

    /// Update the listening ports and return the ports that were removed
    /// from the set for further unmapping of these ports.
    fn update_ports(&mut self, params: &impl PollParameters) -> HashSet<u16> {
        let mut found = HashSet::new();

        for addr in params.listened_addresses() {
            let mut iter = addr.iter();

            let (ip, port) = (iter.next(), iter.next());

            if let (Some(Protocol::Ip4(_)), Some(Protocol::Tcp(port))) =
                (ip, port)
            {
                found.insert(port);
            }
        }

        // Keep ones found and report the difference so they get unmapped
        let diff: HashSet<u16> =
            self.ports.symmetric_difference(&found).copied().collect();

        self.ports = found;

        diff
    }

    /// Update our external IPv4 address by asking IGDs on our network.
    fn update_ip(
        &mut self,
    ) -> Option<(miniupnpc::Urls, miniupnpc::IgdData, Ipv4Addr)> {
        match discover() {
            Ok(Some((urls, data, lanaddr))) => {
                self.observed_addr =
                    miniupnpc::commands::get_external_ip_address(
                        urls.control_url(),
                        data.first().service_type(),
                    )
                    .ok();

                Some((urls, data, lanaddr))
            }
            Ok(None) | Err(_) => None,
        }
    }

    /// Reset the interval timer
    fn reset_timer(&mut self) {
        let delay = Delay::new(Duration::from_secs(Self::INTERVAL));
        self.interval = Some(delay);
    }

    #[inline(always)]
    fn last_report(&mut self) -> Option<NetworkBehaviourAction<Void, Void>> {
        self.pending_actions.pop_front()
    }
}

impl NetworkBehaviour for UpnpBehaviour {
    type ProtocolsHandler = DummyProtocolsHandler;
    type OutEvent = Void;

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        DummyProtocolsHandler::default()
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connected(&mut self, _: &PeerId) {}

    fn inject_disconnected(&mut self, _: &PeerId) {}

    fn inject_event(&mut self, _: PeerId, _: ConnectionId, ev: Void) {
        match ev {}
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Void, Void>> {
        let is_first_poll =
            self.interval.is_none() && self.observed_addr.is_none();
        if is_first_poll {
            self.update_ports(params);

            if self.ports.is_empty() {
                self.reset_timer();
                return Poll::Pending;
            }

            self.update_ip();

            if let Some(ref ip) = self.observed_addr {
                for port in self.ports.iter() {
                    let parts = vec![Protocol::Ip4(*ip), Protocol::Tcp(*port)];
                    let address = Multiaddr::from_iter(parts);

                    trace!(
                        target: "locha-p2p",
                        "UPnP: Observed address {}",
                        address
                    );

                    self.pending_actions.push_back(
                        NetworkBehaviourAction::ReportObservedAddr { address },
                    );
                }

                self.reset_timer();
                return Poll::Ready(self.last_report().unwrap());
            } else {
                self.reset_timer();
                return Poll::Pending;
            }
        }

        // Dispatch pending events first
        if let Some(report) = self.last_report() {
            return Poll::Ready(report);
        }

        // unwrap on interval is fine because is_first_poll was false.
        let interval = self.interval.as_mut().unwrap();
        match Pin::new(interval).poll(cx) {
            Poll::Ready(_) => {
                trace!(
                    target: "locha-p2p",
                    "periodic check of external IP and port mapping"
                );

                // TODO: unmap these ports!
                let _unmap = self.update_ports(params);
                let igd = self.update_ip();

                if let Some(ref ip) = self.observed_addr {
                    for port in self.ports.iter() {
                        if let Some((ref urls, ref data, ref lanaddr)) = igd {
                            let res = miniupnpc::commands::add_port_mapping(
                                urls.control_url(),
                                data.first().service_type(),
                                *port,
                                *port,
                                *lanaddr,
                                &self.protocol_desc,
                                miniupnpc::commands::Protocol::Tcp,
                                None,
                                Duration::from_secs(0),
                            );

                            if res.is_ok() {
                                info!(
                                    target: "locha-p2p",
                                    "UPnP: Port {} mapped successfully",
                                    port
                                );
                            }
                        }

                        let parts =
                            vec![Protocol::Ip4(*ip), Protocol::Tcp(*port)];
                        let address = Multiaddr::from_iter(parts);

                        trace!(
                            target: "locha-p2p",
                            "UPnP: Observed address {}",
                            address
                        );

                        self.pending_actions.push_back(
                            NetworkBehaviourAction::ReportObservedAddr {
                                address,
                            },
                        );
                    }
                }

                self.reset_timer();

                if let Some(report) = self.last_report() {
                    return Poll::Ready(report);
                }
            }
            Poll::Pending => (),
        }

        Poll::Pending
    }
}

fn discover() -> Result<
    Option<(miniupnpc::Urls, miniupnpc::IgdData, Ipv4Addr)>,
    miniupnpc::Error,
> {
    match miniupnpc::discover(
        Duration::from_secs(2),
        None,
        None,
        miniupnpc::LocalPort::Any,
        false,
        2,
    )
    .map(|list| list.get_valid_igd())?
    {
        Some(igd) => {
            if igd.data.is_none() {
                error!(target: "locha-p2p", "UPnP: No valid IGDs found");
                return Ok(None);
            }

            Ok(Some((igd.urls, igd.data.unwrap(), igd.lan_address)))
        }
        None => Ok(None),
    }
}
