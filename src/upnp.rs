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

use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::time::Duration;

use futures::channel::mpsc::{channel, Receiver, SendError, Sender};
use futures::channel::oneshot::channel as oneshot_channel;
use futures::channel::oneshot::Sender as OneshotSender;
use futures::{Future, FutureExt, SinkExt, StreamExt};

use log::error;

pub use miniupnpc::commands::Protocol;

pub const CHECK_PORT_INTERVALS_SECS: u64 = 1200;

#[derive(Debug, Clone)]
pub struct Upnp {
    tx: Sender<UpnpEvent>,
}

impl Upnp {
    pub fn new() -> (Upnp, impl Future<Output = ()> + Send + 'static) {
        let (tx, rx) = channel(5);
        (Upnp { tx }, task(rx))
    }

    pub async fn get_external_ip_address(
        &self,
    ) -> Result<Option<Ipv4Addr>, SendError> {
        let (tx, rx) = oneshot_channel();
        self.tx
            .clone()
            .send(UpnpEvent::GetExternalIPAddress(tx))
            .await?;

        Ok(rx.await.unwrap())
    }

    pub async fn add_port_mapping(
        &self,
        desc: String,
        proto: Protocol,
        port: u16,
    ) -> Result<(), SendError> {
        self.tx
            .clone()
            .send(UpnpEvent::AddPortMapping(PortMap { desc, proto, port }))
            .await
    }

    pub async fn stop(&self) -> Result<(), SendError> {
        self.tx.clone().send(UpnpEvent::Stop).await
    }
}

#[derive(Debug)]
enum UpnpEvent {
    Stop,
    GetExternalIPAddress(OneshotSender<Option<Ipv4Addr>>),
    AddPortMapping(PortMap),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PortMap {
    desc: String,
    proto: Protocol,
    port: u16,
}

async fn task(mut rx: Receiver<UpnpEvent>) {
    let mut port_mappings: HashSet<PortMap> = HashSet::new();
    let mut interval = async_std::stream::interval(Duration::from_secs(
        CHECK_PORT_INTERVALS_SECS,
    ));

    loop {
        futures::select_biased! {
            event = rx.next().fuse() => {
                let event = event.unwrap_or(UpnpEvent::Stop);

                match event {
                    UpnpEvent::GetExternalIPAddress(tx) => {
                        tx.send(external_ip()).ok();
                    }
                    UpnpEvent::AddPortMapping(map) => {
                        if port_mappings.get(&map).is_none() {
                            port_mappings.insert(map.clone());
                            match discover() {
                                Ok(Some((urls, igd, lanaddr))) => {
                                    miniupnpc::commands::add_port_mapping(
                                        urls.control_url(),
                                        igd.first().service_type(),
                                        map.port,
                                        map.port,
                                        lanaddr,
                                        map.desc,
                                        map.proto,
                                        None,
                                        Duration::from_secs(0),
                                    )
                                    .ok();
                                }
                                Ok(None) | Err(_) => {}
                            }
                        }
                    }
                    UpnpEvent::Stop => return,
                };
            },
            _ = interval.next().fuse() => {
                // If any port mapping, remap them because we don't know
                // if the map was kept on the UPnP IGD, so... for reliability
                // just remap again.
                //
                // This might actually help in the case the IGD is reset, or
                // any other reason. Doing this each 20 mins (1200 s) does not
                // hurts performance.
                if port_mappings.len() > 0 {
                    match discover() {
                        Ok(Some((urls, igd, lanaddr))) => {
                            for map in port_mappings.iter() {
                                miniupnpc::commands::add_port_mapping(
                                    urls.control_url(),
                                    igd.first().service_type(),
                                    map.port,
                                    map.port,
                                    lanaddr,
                                    &map.desc,
                                    map.proto,
                                    None,
                                    Duration::from_secs(0),
                                )
                                .ok();
                            }
                        }
                        Ok(None) | Err(_) => (),
                    };
                }
            }
        }
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

fn external_ip() -> Option<Ipv4Addr> {
    if let Ok(Some((urls, igd, _))) = discover() {
        miniupnpc::commands::get_external_ip_address(
            urls.control_url(),
            igd.first().service_type(),
        )
        .ok()
    } else {
        None
    }
}

pub mod behaviour {
    use super::external_ip;

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

    use log::trace;

    pub struct UpnpBehaviour {
        interval: Option<Delay>,
        observed_addr: Option<Ipv4Addr>,
        pending_actions: VecDeque<NetworkBehaviourAction<Void, Void>>,
    }

    impl UpnpBehaviour {
        pub fn new() -> UpnpBehaviour {
            UpnpBehaviour {
                interval: None,
                observed_addr: None,
                pending_actions: VecDeque::new(),
            }
        }
    }

    impl Default for UpnpBehaviour {
        fn default() -> Self {
            Self::new()
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
            fn guess_ports(params: &impl PollParameters) -> HashSet<u16> {
                let mut ports = HashSet::new();

                for addr in params.listened_addresses() {
                    let mut iter = addr.iter();

                    let (ip, port) = (iter.next(), iter.next());

                    if let (Some(Protocol::Ip4(_)), Some(Protocol::Tcp(port))) =
                        (ip, port)
                    {
                        ports.insert(port);
                    }
                }

                trace!(target: "locha-p2p", "ports: {:?}", ports);

                ports
            }

            // Dispatch pending events first
            if let Some(action) = self.pending_actions.pop_front() {
                return Poll::Ready(action);
            }

            if self.interval.is_none() && self.observed_addr.is_none() {
                let ports = guess_ports(params);
                if ports.is_empty() {
                    self.interval = Some(Delay::new(Duration::from_secs(1200)));
                    return Poll::Pending;
                }

                self.observed_addr = external_ip();
                self.interval = Some(Delay::new(Duration::from_secs(1200)));

                if let Some(ref ip) = self.observed_addr {
                    for port in ports {
                        let parts =
                            vec![Protocol::Ip4(*ip), Protocol::Tcp(port)];
                        let address = Multiaddr::from_iter(parts);

                        trace!(
                            target: "locha-p2p",
                            "observed address {}",
                            address
                        );

                        self.pending_actions.push_back(
                            NetworkBehaviourAction::ReportObservedAddr {
                                address,
                            },
                        );
                    }

                    return Poll::Ready(
                        self.pending_actions.pop_front().unwrap(),
                    );
                } else {
                    return Poll::Pending;
                }
            }

            let ports = guess_ports(params);
            if self.interval.is_some() && !ports.is_empty() {
                let interval = self.interval.as_mut().unwrap();

                match Pin::new(interval).poll(cx) {
                    Poll::Ready(_) => {
                        trace!(target: "locha-p2p", "periodic check of external IP");
                        let addr = external_ip();

                        if addr.is_some() && self.observed_addr.is_some() {
                            let new = addr.as_ref().unwrap();
                            let old = addr.as_ref().unwrap();

                            if new != old {
                                self.observed_addr = addr;
                            }
                        }

                        if let Some(ref ip) = self.observed_addr {
                            for port in ports {
                                let parts = vec![
                                    Protocol::Ip4(*ip),
                                    Protocol::Tcp(port),
                                ];
                                let address = Multiaddr::from_iter(parts);

                                self.pending_actions.push_back(
                                    NetworkBehaviourAction::ReportObservedAddr {
                                        address,
                                    },
                                );
                            }
                        }

                        self.interval =
                            Some(Delay::new(Duration::from_secs(1200)));
                    }
                    Poll::Pending => (),
                }
            }

            if let Some(action) = self.pending_actions.pop_front() {
                return Poll::Ready(action);
            }

            Poll::Pending
        }
    }
}
