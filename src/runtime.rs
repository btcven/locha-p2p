// Copyright 2020 Locha Inc
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
//! use locha_p2p::Multiaddr;
//!
//! struct EventsHandler;
//!
//! impl RuntimeEvents for EventsHandler {
//!     fn on_new_message(&mut self, message: String) {
//!         println!("new message: {}", message);
//!     }
//!
//!     fn on_new_listen_addr(&mut self, addr: Multiaddr) {
//!         println!("new listen addr: {}", addr);
//!     }
//! }
//!
//! let config = RuntimeConfig {
//!     identity: Identity::generate(),
//!     listen_addr: "/ip4/0.0.0.0/tcp/0".parse().expect("invalid address"),
//!     channel_cap: 20,
//!     heartbeat_interval: 5,
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
pub mod sync_start_cond;

use std::io;
use std::time::Duration;

use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

use futures::prelude::*;
use futures::select;

use libp2p::core::connection::{ConnectedPoint, ConnectionError};
use libp2p::swarm::protocols_handler::NodeHandlerWrapperError;
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::Transport;

use libp2p::gossipsub::GossipsubEvent;
use libp2p::gossipsub::{Gossipsub, GossipsubConfigBuilder, MessageId};
use libp2p::gossipsub::{GossipsubMessage, MessageAuthenticity, Topic};

use libp2p::identity::Keypair;
use libp2p::{Multiaddr, PeerId};

use log::{error, info, trace, warn};

use self::config::RuntimeConfig;
use self::error::Error;
use self::events::RuntimeEvents;
use self::sync_start_cond::{StartStatus, SyncStartCond};

use crate::identity::Identity;

/// Gossipsub protocol name for Locha P2P Chat
pub const CHAT_SERVICE_GOSSIP_PROTCOL_NAME: &[u8] = b"/locha-gossip/1.0.0";

/// Locha P2P runtime
pub struct Runtime {
    handle: Option<task::JoinHandle<Result<(), Error>>>,
    tx: Option<Sender<ChatAction>>,

    identity: Option<Identity>,
}

impl Runtime {
    /// Create a new runtime
    pub fn new() -> Runtime {
        trace!("creating new Runtime");

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
        trace!("starting chat service");

        if self.is_started() {
            warn!("chat service is already started");
            return Err(Error::AlreadyStarted);
        }

        let (tx, rx) = channel::<ChatAction>(config.channel_cap);

        let identity = config.identity.clone();

        let cond = SyncStartCond::new();
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
        trace!("stopping chat service");

        if !self.is_started() {
            error!("chat service is not started");
            return Err(Error::NotStarted);
        }

        if self.handle.is_none() {
            self.tx = None;
            return Ok(());
        }

        // Send Stop action and wait for thread to finish.
        self.send_action(ChatAction::Stop)?;
        task::block_on(async { self.handle.as_mut().unwrap().await })?;

        self.handle = None;
        self.tx = None;

        Ok(())
    }

    /// Dial a peer using it's multiaddress
    pub fn dial(&self, multiaddr: Multiaddr) -> Result<(), Error> {
        trace!("sending dial: {}", multiaddr);

        self.send_action(ChatAction::Dial(multiaddr))
    }

    /// Send a message
    pub fn send_message(&self, message: String) -> Result<(), Error> {
        trace!("sending message");

        self.send_action(ChatAction::SendMessage(message))
    }

    /// Send an action to the event loop.
    fn send_action(&self, action: ChatAction) -> Result<(), Error> {
        if self.tx.is_none() {
            if self.handle.is_none() {
                error!("Runtime is not initialized");
            }

            return Err(Error::ChannelClosed);
        }

        task::block_on(async { self.tx.as_ref().unwrap().send(action).await });
        Ok(())
    }

    /// Builds the transport we're going to use
    fn build_transport(
        keypair: Keypair,
    ) -> std::io::Result<
        impl Transport<
                Output = (
                    PeerId,
                    impl libp2p::core::muxing::StreamMuxer<
                            OutboundSubstream = impl Send,
                            Substream = impl Send,
                            Error = impl Into<std::io::Error>,
                        > + Send
                        + Sync,
                ),
                Error = impl std::error::Error + Send,
                Listener = impl Send,
                Dial = impl Send,
                ListenerUpgrade = impl Send,
            > + Clone,
    > {
        use libp2p::core::upgrade::{SelectUpgrade, Version};
        use libp2p::dns::DnsConfig;
        use libp2p::noise;
        use libp2p::tcp::TcpConfig;
        use libp2p::websocket::WsConfig;

        // Create our low level TCP transport, and on top of it create a
        // WebSockets transport. They can be used both at the same time.
        let tcp = TcpConfig::new().nodelay(true);
        let dns = DnsConfig::new(tcp)?;
        let ws = WsConfig::new(dns.clone());
        let transport = dns.or_transport(ws);

        // Use the noise protocol to handle encryption and negotiation.
        // Also we use yamux and mplex to multiplex connections to peers.
        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&keypair)
            .expect("Signing noise static DH keypair failed.");

        Ok(transport
            .upgrade(Version::V1)
            .authenticate(
                noise::NoiseConfig::xx(noise_keys).into_authenticated(),
            )
            .multiplex(SelectUpgrade::new(
                libp2p::yamux::Config::default(),
                libp2p::mplex::MplexConfig::default(),
            ))
            .map(|(peer, muxer), _| {
                (peer, libp2p::core::muxing::StreamMuxerBox::new(muxer))
            })
            .timeout(Duration::from_secs(20)))
    }

    fn message_id(message: &GossipsubMessage) -> MessageId {
        use libp2p::multihash::{Keccak384, MultihashDigest};

        let mut hasher = Keccak384::default();
        hasher.input(message.data.as_slice());

        MessageId::from(hasher.result().into_bytes())
    }

    /// Main event loop of the Chat Service. This is where we handle all logic
    /// from libp2p and the network behaviour and also we handle our own actions
    /// as sending a message or dialing a node.
    async fn event_loop(
        cond: SyncStartCond,
        rx: Receiver<ChatAction>,
        config: RuntimeConfig,
        mut events_handler: Box<dyn RuntimeEvents>,
    ) -> Result<(), Error> {
        let transport = Self::build_transport(config.identity.keypair())?;

        let gossipsub_config = GossipsubConfigBuilder::new()
            .protocol_id(CHAT_SERVICE_GOSSIP_PROTCOL_NAME)
            .heartbeat_interval(Duration::from_secs(config.heartbeat_interval))
            .message_id_fn(Self::message_id)
            .build();

        // Create a Gossipsub topic
        // TODO: Make topics dynamic per peer
        let topic = Topic::new("locha-p2p-testnet".into());

        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(config.identity.keypair()),
            gossipsub_config,
        );
        gossipsub.subscribe(topic.clone());

        let mut swarm = Swarm::new(transport, gossipsub, config.identity.id());
        match Swarm::listen_on(&mut swarm, config.listen_addr.clone()) {
            Ok(_) => (),
            Err(e) => {
                error!("Could not listen on {}: {}", config.listen_addr, e);
                cond.notify_failure();
                return Err(e.into());
            }
        }

        cond.notify_start();

        loop {
            select! {
                action = rx.recv().fuse() => {
                    if action.is_err() {
                        warn!("Channel has been dropped without exiting properly");
                        break;
                    }

                    let action = action.unwrap();

                    match action {
                        ChatAction::Stop => {
                            info!("Stopping chat service");
                            break;
                        },
                        ChatAction::Dial(to_dial) => {
                            info!("Dialing address: {}", to_dial);
                            if let Err(e) = Swarm::dial_addr(&mut swarm, to_dial.clone()) {
                                error!("dial to {} failed: {}", to_dial, e);
                            }
                        }
                        ChatAction::SendMessage(message) => {
                            match swarm.publish(&topic, message.as_bytes()) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("couldn't send message: {:?}", e);
                                }
                            }
                        }
                    }
                },
                event = swarm.next_event().fuse() => Self::handle_swarm_event(&event, &mut *events_handler).await,
            }
        }

        Ok(())
    }

    async fn handle_swarm_event(
        swarm_event: &SwarmEvent<GossipsubEvent, io::Error>,
        events_handler: &mut dyn RuntimeEvents,
    ) {
        trace!("new swarm event");

        match *swarm_event {
            SwarmEvent::Behaviour(ref behaviour) => {
                Self::handle_gossipsub_event(behaviour, events_handler).await
            }
            SwarmEvent::ConnectionEstablished {
                ref peer_id,
                ref endpoint,
                ref num_established,
            } => {
                log_connection_established(
                    &peer_id,
                    &endpoint,
                    num_established.get(),
                );
            }
            SwarmEvent::ConnectionClosed {
                ref peer_id,
                ref endpoint,
                ref num_established,
                ref cause,
            } => {
                log_connection_closed(
                    &peer_id,
                    &endpoint,
                    *num_established,
                    cause,
                );
            }
            SwarmEvent::IncomingConnection {
                ref local_addr,
                ref send_back_addr,
            } => {
                info!("Incoming connection on {}, with protocols for sending back {}",
                      local_addr, send_back_addr);
            }
            SwarmEvent::IncomingConnectionError {
                ref local_addr,
                ref send_back_addr,
                ref error,
            } => {
                error!(
                    "Incoming connection error on {}: {}\n\
                       Protocols for sending back {}",
                    local_addr, error, send_back_addr
                );
            }
            SwarmEvent::BannedPeer { ref peer_id, .. } => {
                info!("Peer {} is banned", peer_id);
            }
            SwarmEvent::UnreachableAddr {
                ref peer_id,
                ref address,
                ref error,
                ref attempts_remaining,
            } => {
                warn!(
                    "Address {} for peer {} is unreachable.\n\
                      Attempt failed with error {}.\n\
                      Attempts remaining: {}",
                    peer_id, address, error, attempts_remaining
                );
            }
            SwarmEvent::UnknownPeerUnreachableAddr {
                ref address,
                ref error,
            } => {
                warn!(
                    "Unknown peer address {} is unreachable.\n\
                       Attempt failed with error {}",
                    address, error
                );
            }
            SwarmEvent::NewListenAddr(ref address) => {
                info!("Listening on new address {}", address);
                events_handler.on_new_listen_addr(address.clone())
            }
            SwarmEvent::ExpiredListenAddr(ref address) => {
                info!("Listening address {} expired", address);
            }
            SwarmEvent::ListenerClosed {
                ref addresses,
                ref reason,
            } => {
                let mut affected = String::new();
                for address in addresses {
                    affected.push_str(format!(", {}\n", address).as_str())
                }

                match *reason {
                    Ok(_) => {
                        warn!(
                            "Listener closed.\n\
                            Addresses affected {}",
                            affected,
                        );
                    }
                    Err(ref e) => {
                        warn!(
                            "Listener closed. Reason {}.\n\
                            Addresses affected:\n{}",
                            e, affected,
                        );
                    }
                }
            }
            SwarmEvent::ListenerError { ref error } => {
                warn!("Listener error {}.", error);
            }
            SwarmEvent::Dialing(ref peer_id) => {
                info!("Dialing peer {}", peer_id);
            }
        }
    }

    /// Handle gossipsub events
    async fn handle_gossipsub_event(
        event: &GossipsubEvent,
        events_handler: &mut dyn RuntimeEvents,
    ) {
        if let GossipsubEvent::Message(ref peer_id, ref id, ref message) =
            *event
        {
            info!("Received message {}, from peer {}", id, peer_id);

            let contents =
                String::from_utf8_lossy(message.data.as_slice()).into_owned();
            events_handler.on_new_message(contents);
        }
    }
}

impl Default for Runtime {
    fn default() -> Runtime {
        Self::new()
    }
}

/// Log when a connection is established to a peer.
fn log_connection_established(
    peer_id: &PeerId,
    endpoint: &ConnectedPoint,
    num_established: u32,
) {
    match endpoint {
        ConnectedPoint::Dialer { address } => {
            info!(
                "Outbound connection to peer {} on address {} succeed.\n\
                  Total number of established connections to peer are {}",
                peer_id, address, num_established
            );
        }
        ConnectedPoint::Listener {
            local_addr,
            send_back_addr,
        } => {
            info!("Inbound connection to peer {} established on our address {} succeed.\n\
                  The stack of protocols for sending back to this peer are {}.\n\
                  Total number of established connections to this peer are {}",
                  peer_id, local_addr, send_back_addr, num_established);
        }
    }
}

/// Log when a connection to a peer is closed.
fn log_connection_closed(
    peer_id: &PeerId,
    endpoint: &ConnectedPoint,
    num_established: u32,
    cause: &Option<ConnectionError<NodeHandlerWrapperError<io::Error>>>,
) {
    match endpoint {
        ConnectedPoint::Dialer { address } => {
            match cause {
                Some(cause) => {
                    info!("Outbound connection to peer {} on address {} failed.\n\
                          The cause of the close is {}.\n\
                          The number of remaining connections to peer {}",
                          peer_id, address, cause,
                          num_established);
                }
                None => {
                    info!("Outbound connection to peer {} on address {} failed.\n\
                          The number of remaining connections to peer are {}",
                          address, peer_id, num_established);
                }
            }
        }
        ConnectedPoint::Listener {
            local_addr,
            send_back_addr,
        } => match cause {
            Some(cause) => {
                info!("Inbound connection from peer {} on our local address {} failed.\n\
                          The cause of the close is {}.\n\
                          The stack of protocols for sending back for this peer are {}.\n\
                          The number of remaining connections to peer are {}",
                          peer_id, local_addr, cause, send_back_addr, num_established);
            }
            None => {
                info!("Inbound connection from peer {} on our local address {} failed.\n\
                          The stack of protocols for sending back for this peer are {}.\n\
                          The number of remaining connections to peer are {}",
                          peer_id, local_addr, send_back_addr, num_established);
            }
        },
    }
}

/// Chat service action
enum ChatAction {
    /// Send a message
    SendMessage(String),
    /// Dial a peer
    Dial(Multiaddr),
    /// Stop the chat service
    Stop,
}
