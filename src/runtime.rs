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
//! use locha_p2p::identity::Identity;
//! use locha_p2p::runtime::Runtime;
//! use locha_p2p::runtime::events::RuntimeEvents;
//! use locha_p2p::runtime::config::RuntimeConfig;
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
//! let config = RuntimeConfig {
//!     identity: Identity::generate(),
//!     listen_addr: "/ip4/0.0.0.0/tcp/0".parse().expect("invalid address"),
//!     channel_cap: 20,
//!     heartbeat_interval: 5,
//!
//!     // Yes, allow discovery of private IPv4 adddresses
//!     allow_ipv4_private: true,
//!     allow_ipv4_shared: false,
//!     allow_ipv6_link_local: true,
//!     // Allow discovery of IPv6 unique local addresses which are used
//!     // by Locha Mesh, cjdns and private networks not reachable on the
//!     // public internet.
//!     allow_ipv6_ula: true,
//!     // Allow discovery through mDNS
//!     use_mdns: true,
//! };
//!
//! let mut runtime = Runtime::new();
//!
//! runtime.start(config, Box::new(EventsHandler)).expect("could not start runtime");
//!
//! // Send a message and the runtime will dispatch it.
//! runtime.send_message("Welcome, bienvenido!".to_string()).expect("could not send message");
//!
//! // Can be stopped at any time when requested.
//! runtime.stop().expect("runtime failed to stop or has been already stopped");
//! ```

pub mod config;
pub mod error;
pub mod events;

use std::io;
use std::iter::FromIterator;
use std::time::Duration;

use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

use futures::prelude::*;
use futures::select;

use libp2p::swarm::{NetworkBehaviour, SwarmEvent};

use libp2p::core::either::EitherError;

use libp2p::Multiaddr;

use log::{debug, error, info, trace, warn};

use self::config::RuntimeConfig;
use self::error::Error;
use self::events::RuntimeEvents;

use crate::discovery::DiscoveryEvent;
use crate::gossip::{GossipsubEvent, Topic};
use crate::network::{Network, NetworkEvent};

use crate::sync::{StartCond, StartStatus};

use crate::identity::Identity;
use crate::Swarm;

/// Gossipsub protocol name for Locha P2P Chat
pub const CHAT_SERVICE_GOSSIP_PROTCOL_NAME: &[u8] = b"/locha-gossip/1.0.0";

/// Locha P2P runtime
pub struct Runtime {
    handle: Option<task::JoinHandle<Result<(), Error>>>,
    tx: Option<Sender<RuntimeAction>>,

    identity: Option<Identity>,
}

impl Runtime {
    /// Create a new runtime
    pub fn new() -> Runtime {
        trace!(target: "locha-p2p", "creating new Runtime");

        Runtime {
            handle: None,
            tx: None,

            identity: None,
        }
    }

    /// Has been started this Runtime?
    pub fn is_started(&self) -> bool {
        self.handle.is_some() && self.tx.is_some()
    }

    /// Identity of the Runtime, this is the Peer (this node)
    /// identity.
    pub fn identity(&self) -> &Identity {
        &self
            .identity
            .as_ref()
            .expect("chat service has not been started")
    }

    /// Start the runtime with the provided configuration and events
    /// handler.
    pub fn start(
        &mut self,
        config: RuntimeConfig,
        events_handler: Box<dyn RuntimeEvents>,
    ) -> Result<(), Error> {
        trace!(target: "locha-p2p", "starting chat service");

        if self.is_started() {
            warn!(target: "locha-p2p", "chat service is already started");
            return Err(Error::AlreadyStarted);
        }

        let (tx, rx) = channel::<RuntimeAction>(config.channel_cap);

        let identity = config.identity.clone();

        let cond = StartCond::new();
        let handle = task::spawn({
            let cond = cond.clone();

            async { Self::event_loop(cond, rx, config, events_handler).await }
        });
        if let StartStatus::Failed = cond.wait() {
            return task::block_on(async move { handle.await });
        }

        self.handle = Some(handle);
        self.tx = Some(tx);

        self.identity = Some(identity);

        Ok(())
    }

    /// Stop the runtime. This function will block until the runtime
    /// is closed.
    pub fn stop(&mut self) -> Result<(), Error> {
        debug!(target: "locha-p2p", "stopping chat service");

        if !self.is_started() {
            error!(target: "locha-p2p", "chat service is not started");
            return Err(Error::NotStarted);
        }

        if self.handle.is_none() {
            self.tx = None;
            return Ok(());
        }

        // Send Stop action and wait for thread to finish.
        self.send_action(RuntimeAction::Stop)?;
        task::block_on(async { self.handle.as_mut().unwrap().await })?;

        self.handle = None;
        self.tx = None;

        Ok(())
    }

    /// Dial a peer using it's multiaddress
    pub fn dial(&self, multiaddr: Multiaddr) -> Result<(), Error> {
        trace!(target: "locha-p2p", "sending dial: {}", multiaddr);

        self.send_action(RuntimeAction::Dial(multiaddr))
    }

    /// Send a message
    pub fn send_message(&self, message: String) -> Result<(), Error> {
        trace!(target: "locha-p2p", "sending message");

        self.send_action(RuntimeAction::SendMessage(message))
    }

    /// Send an action to the event loop.
    fn send_action(&self, action: RuntimeAction) -> Result<(), Error> {
        if self.tx.is_none() {
            if self.handle.is_none() {
                error!(target: "locha-p2p", "Runtime is not initialized");
            }

            return Err(Error::ChannelClosed);
        }

        task::block_on(async { self.tx.as_ref().unwrap().send(action).await });
        Ok(())
    }

    /// Main event loop of the Chat Service. This is where we handle all logic
    /// from libp2p and the network behaviour and also we handle our own actions
    /// as sending a message or dialing a node.
    async fn event_loop(
        cond: StartCond,
        rx: Receiver<RuntimeAction>,
        config: RuntimeConfig,
        mut events_handler: Box<dyn RuntimeEvents>,
    ) -> Result<(), Error> {
        let transport = match build_transport(&config.identity.keypair()) {
            Ok(t) => t,
            Err(e) => {
                error!(
                    target: "locha-p2p",
                    "Could not create transport: {}",
                    e
                );
                cond.notify_failure();
                return Err(e.into());
            }
        };

        // Create a Gossipsub topic
        // TODO: Make topics dynamic per peer
        let topic = Topic::new("locha-p2p-testnet".into());
        network.subscribe(topic.clone());

        let mut swarm = Swarm::new(transport, network, config.identity.id());
        match Swarm::listen_on(&mut swarm, config.listen_addr.clone()) {
            Ok(_) => (),
            Err(e) => {
                error!(
                    target: "locha-p2p",
                    "Could not listen on {}: {}",
                    config.listen_addr, e
                );
                cond.notify_failure();
                return Err(e.into());
            }
        }

        // Signal the calling thread we already started.
        cond.notify_start();

        loop {
            select! {
                action = rx.recv().fuse() => {
                    if action.is_err() {
                        warn!(
                            target: "locha-p2p",
                            "Channel has been dropped without asking to stop"
                        );
                        break;
                    }

                    let action = action.unwrap();

                    match action {
                        RuntimeAction::Stop => {
                            info!(target: "locha-p2p", "Stopping chat service");
                            break;
                        },
                        RuntimeAction::Dial(to_dial) => {
                            debug!(
                                target: "locha-p2p",
                                "Dialing address: {}",
                                to_dial
                            );

                            if let Err(e) = Swarm::dial_addr(&mut swarm, to_dial.clone()) {
                                error!(
                                    target: "locha-p2p",
                                    "dial to {} failed: {}",
                                    to_dial, e
                                );
                            }
                        }
                        RuntimeAction::SendMessage(message) => {
                            match swarm.publish(&topic, message.as_bytes()) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("couldn't send message: {:?}", e);
                                }
                            }
                        }
                    }
                },
                event = swarm.next_event().fuse() => {
                    Self::handle_swarm_event(
                        &mut swarm,
                        &event,
                        &mut *events_handler
                    ).await
                },
            }
        }

        Ok(())
    }

    async fn handle_swarm_event(
        swarm: &mut Swarm,
        swarm_event: &SwarmEvent<
            NetworkEvent,
            EitherError<io::Error, io::Error>,
        >,
        events_handler: &mut dyn RuntimeEvents,
    ) {
        trace!(target: "locha-p2p", "new swarm event");

        match *swarm_event {
            SwarmEvent::Behaviour(ref behaviour) => {
                Self::handle_behaviour_event(swarm, behaviour, events_handler)
                    .await
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
                events_handler
                    .on_incomming_connection(local_addr, send_back_addr);
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

    /// Handle gossipsub events
    async fn handle_behaviour_event(
        swarm: &mut Swarm,
        event: &NetworkEvent,
        events_handler: &mut dyn RuntimeEvents,
    ) {
        match *event {
            NetworkEvent::Gossipsub(ref gossip_ev) => {
                if let GossipsubEvent::Message(
                    ref _peer,
                    ref _id,
                    ref message,
                ) = **gossip_ev
                {
                    let contents =
                        String::from_utf8_lossy(message.data.as_slice())
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
}

impl Default for Runtime {
    fn default() -> Runtime {
        Self::new()
    }
}

/// Runtime action
enum RuntimeAction {
    /// Send a message
    SendMessage(String),
    /// Dial a peer
    Dial(Multiaddr),
    /// Stop the chat service
    Stop,
}
