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

//! # UPnP network behaviour
//!
//! This network behaviour is in charge of finding UPnP Internet Gateway
//! Devices to ask these devices about our Internet connection status and
//! our IPv4 external IP address if we have one.
//!
//! TODO: handle UPnP port mapping

use std::net::Ipv4Addr;
use std::task::{Context, Poll};
use std::time::Duration;

use async_std::pin::Pin;
use async_std::sync::{channel, Receiver};
use async_std::task;

use futures::stream::Stream;

use libp2p::core::connection::ConnectionId;
use libp2p::swarm::protocols_handler::DummyProtocolsHandler;
use libp2p::swarm::{KeepAlive, PollParameters};
use libp2p::swarm::{NetworkBehaviour, NetworkBehaviourAction};
use libp2p::{Multiaddr, PeerId};

use void::Void;

use log::{debug, error, warn};

#[derive(Debug)]
pub struct Upnp {
    worker: task::JoinHandle<()>,
    rx: Receiver<UpnpEvent>,
}

impl Upnp {
    pub fn new(tick: Duration) -> Upnp {
        let (tx, rx) = channel::<UpnpEvent>(10);

        Upnp {
            worker: task::spawn(async move {
                let sleep_time = tick;

                loop {
                    let igd = match miniupnpc::discover(
                        Duration::from_secs(2),
                        None,
                        None,
                        miniupnpc::LocalPort::Any,
                        false,
                        2,
                    )
                    .map(|list| list.get_valid_igd())
                    {
                        Ok(v) if v.is_some() => {
                            let igd = v.unwrap();
                            if igd.data.is_none() {
                                debug!(
                                    target: "locha-p2p",
                                    "found UPnP device(s), but aren't gateways: {:?}",
                                    igd
                                );
                                task::sleep(sleep_time).await;
                                continue;
                            }
                            igd
                        }
                        Ok(_) => {
                            debug!(target: "locha-p2p", "no IGD found");
                            task::sleep(sleep_time).await;
                            continue;
                        }
                        Err(e) => {
                            warn!(
                                target: "locha-p2p",
                                "could not get UPnP IGD device: {}",
                                e
                            );
                            task::sleep(sleep_time).await;
                            continue;
                        }
                    };

                    let igd_data = igd.data.unwrap();
                    match miniupnpc::commands::get_external_ip_address(
                        igd.urls.control_url(),
                        igd_data.first().service_type(),
                    ) {
                        Ok(ipv4) => {
                            debug!(
                                target: "locha-p2p",
                                "found external IPv4 address: {}",
                                ipv4
                            );
                            tx.send(UpnpEvent::ExternalIpv4Address(ipv4)).await;
                        }
                        Err(e) => {
                            warn!(
                                target: "locha-p2p",
                                "External IPv4 address not found: {}",
                                e
                            );
                        }
                    };

                    task::sleep(sleep_time).await;
                }
            }),
            rx,
        }
    }
}

/// UPnP event
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UpnpEvent {
    /// Found our external IPv4 address
    ExternalIpv4Address(Ipv4Addr),
}

impl NetworkBehaviour for Upnp {
    type ProtocolsHandler = DummyProtocolsHandler;
    type OutEvent = UpnpEvent;

    fn new_handler(&mut self) -> DummyProtocolsHandler {
        DummyProtocolsHandler {
            keep_alive: KeepAlive::No,
        }
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        vec![]
    }

    fn inject_connected(&mut self, _: &PeerId) {}
    fn inject_disconnected(&mut self, _: &PeerId) {}

    fn inject_event(&mut self, _: PeerId, _: ConnectionId, event: Void) {
        match event {}
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        _: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Void, UpnpEvent>> {
        match Pin::new(&mut self.rx).poll_next(cx) {
            Poll::Ready(Some(ev)) => {
                Poll::Ready(NetworkBehaviourAction::GenerateEvent(ev))
            }
            Poll::Ready(None) => {
                error!(
                    target: "locha-p2p",
                    "UPnP worker thread has dropped channel TX"
                );
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
