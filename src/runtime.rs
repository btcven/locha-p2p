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

//! # Runtime
//!
//! This contains a runtime for the Chat. The network behaviour and swarm is
//! handled on a separate thread, we communicate with the thread by sending
//! actions to it. It also reports us throhgh callbacks the events that
//! happen. This is especially useful for language bridges such as the JNI
//! or Node.JS where we can't use the async/await method as we do on Rust and
//! we need a thread to do it for us, that's what `Runtime` does.
//!
//! # Examples
//!
//! ```rust
//! use locha_p2p::discovery::DiscoveryConfig;
//! use locha_p2p::identity::Identity;
//! use locha_p2p::runtime::events::RuntimeEvents;
//! use locha_p2p::runtime::config::RuntimeConfig;
//! use locha_p2p::runtime::Runtime;
//! use locha_p2p::upnp::Upnp;
//! use locha_p2p::{Multiaddr, PeerId};
//!
//! struct EventsHandler;
//!
//! impl RuntimeEvents for EventsHandler {
//!     fn on_new_message(&mut self, message: String) {
//!         println!("new message: {}", message);
//!     }
//! }
//!
//! let identity = Identity::generate();
//!
//! let mut discovery = DiscoveryConfig::new();
//!
//! discovery.id(identity.id());
//!
//! let config = RuntimeConfig {
//!     identity,
//!     listen_addr: "/ip4/0.0.0.0/tcp/0".parse().expect("invalid address"),
//!     channel_cap: 20,
//!     heartbeat_interval: 5,
//!
//!     // Yes, allow discovery of private IPv4 adddresses
//!     discovery,
//! };
//!
//! let (runtime, runtime_task) = Runtime::new(config, Box::new(EventsHandler), None).unwrap();
//!
//! async_std::task::spawn(runtime_task);
//!
//! async_std::task::spawn(async move {
//!     // Send a message and the runtime will dispatch it.
//!     runtime.send_message("Welcome, bienvenido!".to_string()).await;
//!
//!     // Can be stopped at any time when requested.
//!     runtime.stop().await;
//! });
//! ```

pub mod config;
pub mod error;
pub mod events;

use futures::channel::mpsc::{channel, Receiver, SendError, Sender};
use futures::channel::oneshot::{
    channel as oneshot_channel, Sender as OneshotSender,
};
use futures::{Future, FutureExt, SinkExt, StreamExt};

use libp2p::swarm::{NetworkBehaviour, SwarmEvent};

use libp2p::multiaddr::Protocol;
use libp2p::Multiaddr;

use log::{error, trace};

use self::config::RuntimeConfig;
use self::error::Error;
use self::events::RuntimeEvents;

use crate::discovery::DiscoveryEvent;
use crate::gossip::{GossipsubEvent, Topic};
use crate::network::NetworkEvent;
use crate::upnp::Upnp;
use crate::{build_swarm, Swarm};

/// Locha P2P runtime
pub struct Runtime {
    tx: Sender<RuntimeAction>,
}

impl Runtime {
    /// Create a runtime for Locha P2P
    ///
    /// This function will return the [`Runtime`] handle, a [`RuntimeState`]
    /// and a [`UpnpFuture`] which needs to be spawned as soon as possible on
    /// an executor.
    ///
    /// # Arguments
    ///
    /// - `config`: The runtime configuration.
    /// - `events_handler`: Events handler of this runtime.
    pub fn new(
        config: RuntimeConfig,
        events_handler: Box<dyn RuntimeEvents>,
    ) -> Result<(Runtime, impl Future<Output = ()> + Send + 'static), Error>
    {
        let mut swarm =
            build_swarm(&config.identity, config.discovery.clone())?;
        // Create a Gossipsub topic
        // TODO: Make topics dynamic per peer
        let topic = Topic::new("locha-p2p-testnet".into());
        swarm.subscribe(topic.clone());

        match Swarm::listen_on(&mut swarm, config.listen_addr.clone()) {
            Ok(_) => (),
            Err(e) => {
                error!(
                    target: "locha-p2p",
                    "Could not listen on {}: {}",
                    config.listen_addr, e
                );
                return Err(e.into());
            }
        }

        let (tx, rx) = channel(config.channel_cap);

        Ok((Runtime { tx }, task(swarm, events_handler, topic, rx)))
    }

    /// Stop the runtime.
    pub async fn stop(&self) {
        trace!(target: "locha-p2p", "stopping runtime");

        // Send Stop action and wait for thread to finish.
        self.tx.clone().send(RuntimeAction::Stop).await.unwrap()
    }

    /// Dial a peer using it's multiaddress
    pub async fn dial(&self, multiaddr: Multiaddr) {
        trace!(target: "locha-p2p", "dialing: {}", multiaddr);

        self.tx
            .clone()
            .send(RuntimeAction::Dial(multiaddr))
            .await
            .unwrap()
    }

    /// Send a message
    pub async fn send_message(&self, message: String) {
        trace!(target: "locha-p2p", "sending message");

        self.tx
            .clone()
            .send(RuntimeAction::SendMessage(message))
            .await
            .unwrap()
    }

    pub async fn enable_upnp(&self, upnp: Upnp) -> Result<(), &'static str> {
        trace!(target: "locha-p2p", "enabling UPnP");

        let (tx, rx) = oneshot_channel::<Result<(), &'static str>>();

        self.tx
            .clone()
            .send(RuntimeAction::EnableUpnp(upnp, tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }
}

/// Runtime action
enum RuntimeAction {
    Stop,
    Dial(Multiaddr),
    SendMessage(String),
    EnableUpnp(Upnp, OneshotSender<Result<(), &'static str>>),
}

async fn task(
    mut swarm: Swarm,
    mut events_handler: Box<dyn RuntimeEvents>,
    topic: Topic,
    mut rx: Receiver<RuntimeAction>,
) {
    let mut upnp = None;

    loop {
        trace!(target: "locha-p2p", "loop");

        futures::select_biased! {
            action = rx.next().fuse() => {
                let action = action.unwrap_or(RuntimeAction::Stop);

                match action {
                    RuntimeAction::Stop => {
                        rx.close();
                        break;
                    }
                    RuntimeAction::Dial(address) => {
                        if let Err(e) = Swarm::dial_addr(&mut swarm, address.clone()) {
                            error!(
                                target: "locha-p2p",
                                "dial to {} failed: {}",
                                address, e
                            );
                        }
                    }
                    RuntimeAction::SendMessage(message) => {
                        if let Err(e) =
                            swarm.publish(&topic.clone(), message.as_bytes())
                        {
                            error!(
                                target: "locha-p2p",
                                "couldn't send message: {:?}",
                                e
                            );
                        }
                    }
                    RuntimeAction::EnableUpnp(upnp_ref, tx) => {
                        if upnp.is_none() {
                            upnp = Some(upnp_ref);

                            let addrs: Vec<Multiaddr> = Swarm::listeners(&swarm)
                                .map(|a| a.clone())
                                .collect();

                            let mut ok = true;
                            'map: for addr in addrs {
                                if check_port_mapping(upnp.as_ref().unwrap(), &addr).await.is_err() {
                                    upnp = None;
                                    ok = false;
                                    break 'map;
                                }
                            }

                            if ok {
                                tx.send(Ok(())).ok();
                            } else {
                                tx.send(Err("UPnP handle is invalid")).ok();
                            }
                        } else {
                            tx.send(Err("UPnP has been already set")).ok();
                        }
                    }
                }
            },
            ev = swarm.next_event().fuse() => {
                handle_event(&mut swarm, &mut upnp, &mut *events_handler, &ev).await;
            },
        }
    }
}

async fn handle_event<THandleErr: std::error::Error>(
    swarm: &mut Swarm,
    upnp: &mut Option<Upnp>,
    events_handler: &mut dyn RuntimeEvents,
    swarm_event: &SwarmEvent<NetworkEvent, THandleErr>,
) {
    match *swarm_event {
        SwarmEvent::Behaviour(ref ev) => {
            handle_behaviour_event(swarm, events_handler, ev);
        }
        SwarmEvent::ConnectionEstablished {
            ref peer_id,
            ref endpoint,
            ref num_established,
        } => {
            events_handler.on_connection_established(
                peer_id,
                endpoint,
                *num_established,
            );
        }
        SwarmEvent::ConnectionClosed {
            ref peer_id,
            ref endpoint,
            ref num_established,
            ref cause,
        } => events_handler.on_connection_closed(
            peer_id,
            endpoint,
            *num_established,
            cause.as_ref().map(|e| e.to_string()),
        ),
        SwarmEvent::IncomingConnection {
            ref local_addr,
            ref send_back_addr,
        } => {
            events_handler.on_incomming_connection(local_addr, send_back_addr);
        }
        SwarmEvent::IncomingConnectionError {
            ref local_addr,
            ref send_back_addr,
            ref error,
        } => {
            events_handler.on_incomming_connection_error(
                local_addr,
                send_back_addr,
                error,
            );
        }
        SwarmEvent::BannedPeer {
            ref peer_id,
            ref endpoint,
        } => {
            events_handler.on_banned_peer(peer_id, endpoint);
        }
        SwarmEvent::UnreachableAddr {
            ref peer_id,
            ref address,
            ref error,
            ref attempts_remaining,
        } => {
            events_handler.on_unreachable_addr(
                peer_id,
                address,
                error,
                *attempts_remaining,
            );
        }
        SwarmEvent::UnknownPeerUnreachableAddr {
            ref address,
            ref error,
        } => {
            events_handler.on_unknown_peer_unreachable_addr(address, error);
        }
        SwarmEvent::NewListenAddr(ref address) => {
            if let Some(u) = upnp {
                if check_port_mapping(u, address).await.is_err() {
                    // Receiver was dropped, so we'll drop the handle to it.
                    *upnp = None;
                }
            }
            events_handler.on_new_listen_addr(address)
        }
        SwarmEvent::ExpiredListenAddr(ref address) => {
            events_handler.on_expired_listen_addr(address);
        }
        SwarmEvent::ListenerClosed {
            ref addresses,
            ref reason,
        } => {
            events_handler.on_listener_closed(addresses.as_slice(), reason);
        }
        SwarmEvent::ListenerError { ref error } => {
            events_handler.on_listener_error(error);
        }
        SwarmEvent::Dialing(ref peer) => {
            events_handler.on_dialing(peer);
        }
    }
}

fn handle_behaviour_event(
    swarm: &mut Swarm,
    events_handler: &mut dyn RuntimeEvents,
    event: &NetworkEvent,
) {
    match *event {
        NetworkEvent::Gossipsub(ref gossip_ev) => {
            if let GossipsubEvent::Message(ref _peer, ref _id, ref message) =
                **gossip_ev
            {
                let contents = String::from_utf8_lossy(message.data.as_slice())
                    .into_owned();
                events_handler.on_new_message(contents);
            }
        }
        NetworkEvent::Discovery(ref disc_ev) => match *disc_ev {
            DiscoveryEvent::Discovered(ref peer) => {
                let addrs = swarm.addresses_of_peer(peer);
                events_handler.on_peer_discovered(peer, addrs);
            }
            DiscoveryEvent::UnroutablePeer(ref peer) => {
                events_handler.on_peer_unroutable(peer);
            }
        },
    }
}

async fn check_port_mapping(
    upnp: &Upnp,
    address: &Multiaddr,
) -> Result<(), SendError> {
    use crate::upnp::Protocol as UpnpProtocol;

    trace!(target: "locha-p2p", "checking port mapping for {}", address);

    let mut iter = address.iter();
    let ip = iter.next();
    let proto = iter.next();

    let (port, proto) = match (ip, proto) {
        (Some(Protocol::Ip4(_)), Some(Protocol::Tcp(port))) => {
            (port, UpnpProtocol::Tcp)
        }
        (Some(Protocol::Ip4(_)), Some(Protocol::Udp(port))) => {
            (port, UpnpProtocol::Udp)
        }
        _ => return Ok(()),
    };

    // Add port mapping using UPnP.
    upnp.add_port_mapping(
        format!("Locha P2P {}", env!("CARGO_PKG_VERSION")),
        proto,
        port,
    )
    .await
}
