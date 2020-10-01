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
//! use locha_p2p::runtime::events::RuntimeEvents;
//! use locha_p2p::runtime::config::RuntimeConfig;
//! use locha_p2p::runtime::Runtime;
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
//! let config = RuntimeConfig {
//!     identity,
//!     listen_addr: "/ip4/0.0.0.0/tcp/0".parse().expect("invalid address"),
//!     channel_cap: 20,
//!     heartbeat_interval: 5,
//!
//!     upnp: false,
//!     mdns: false,
//!
//!     bootstrap_nodes: Vec::new(),
//! };
//!
//! let (runtime, runtime_task) = Runtime::new(config, Box::new(EventsHandler)).unwrap();
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

pub use libp2p::core::network::NetworkInfo;
pub use libp2p::kad::kbucket::{EntryView, Key};
pub use libp2p::kad::Addresses;

use libp2p::swarm::SwarmEvent;

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::channel::oneshot::{
    channel as oneshot_channel, Sender as OneshotSender,
};
use futures::{Future, FutureExt, SinkExt, StreamExt};

use self::config::RuntimeConfig;
use self::error::Error;
use self::events::RuntimeEvents;

use crate::p2p::pubsub::Topic;
use crate::p2p::{build_swarm, BehaviourEvent, BehaviourEventStream, Swarm};
use crate::{Multiaddr, PeerId};

/// Locha P2P runtime
#[derive(Clone)]
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
        // TODO: fine tune events channel size
        let (mut swarm, event_stream) = build_swarm(
            &config.identity,
            config.upnp,
            config.mdns,
            20,
            config.bootstrap_nodes.iter(),
        )?;
        // Create a Gossipsub topic
        // TODO: Make topics dynamic per peer
        let topic = Topic::new("locha-p2p-testnet".into());
        swarm.subscribe(topic.clone());

        match Swarm::listen_on(&mut swarm, config.listen_addr.clone()) {
            Ok(_) => (),
            Err(e) => {
                log::error!(
                    target: "locha-p2p",
                    "Could not listen on {}: {}",
                    config.listen_addr, e
                );
                return Err(e.into());
            }
        }

        let (tx, rx) = channel(config.channel_cap);

        Ok((
            Runtime { tx },
            task(swarm, event_stream, events_handler, topic, rx),
        ))
    }

    /// Start bootstrapping
    pub async fn bootstrap(&self) {
        log::trace!(target: "locha-p2p", "starting bootstrap");

        self.tx
            .clone()
            .send(RuntimeAction::Bootstrap)
            .await
            .unwrap()
    }

    pub async fn kbuckets(
        &self,
    ) -> Vec<Vec<EntryView<Key<PeerId>, Addresses>>> {
        log::trace!(target: "locha-p2p", "retrieving kbuckets");

        let (tx, rx) =
            oneshot_channel::<Vec<Vec<EntryView<Key<PeerId>, Addresses>>>>();
        self.tx
            .clone()
            .send(RuntimeAction::Kbuckets(tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }

    pub async fn network_info(&self) -> NetworkInfo {
        log::trace!(target: "locha-p2p", "getting network information");

        let (tx, rx) = oneshot_channel::<NetworkInfo>();

        self.tx
            .clone()
            .send(RuntimeAction::NetworkInfo(tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }

    /// Stop the runtime.
    pub async fn stop(&self) {
        log::trace!(target: "locha-p2p", "stopping runtime");

        // Send Stop action and wait for thread to finish.
        self.tx.clone().send(RuntimeAction::Stop).await.unwrap()
    }

    /// Dial a peer using it's multiaddress
    pub async fn dial(&self, multiaddr: Multiaddr) {
        log::trace!(target: "locha-p2p", "dialing: {}", multiaddr);

        self.tx
            .clone()
            .send(RuntimeAction::Dial(multiaddr))
            .await
            .unwrap()
    }

    /// Send a message
    pub async fn send_message(&self, message: String) {
        log::trace!(target: "locha-p2p", "sending message");

        self.tx
            .clone()
            .send(RuntimeAction::SendMessage(message))
            .await
            .unwrap()
    }

    pub async fn external_addresses(&self) -> Vec<Multiaddr> {
        log::trace!(target: "locha-p2p", "getting external addresses");

        let (tx, rx) = oneshot_channel::<Vec<Multiaddr>>();

        self.tx
            .clone()
            .send(RuntimeAction::ExternalAddresses(tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }

    pub async fn peer_id(&self) -> PeerId {
        log::trace!(target: "locha-p2p", "getting peer ID");

        let (tx, rx) = oneshot_channel::<PeerId>();
        self.tx
            .clone()
            .send(RuntimeAction::PeerId(tx))
            .await
            .unwrap();

        rx.await.unwrap()
    }
}

/// Runtime action
enum RuntimeAction {
    Bootstrap,
    Kbuckets(OneshotSender<Vec<Vec<EntryView<Key<PeerId>, Addresses>>>>),
    NetworkInfo(OneshotSender<NetworkInfo>),
    Stop,
    Dial(Multiaddr),
    SendMessage(String),
    ExternalAddresses(OneshotSender<Vec<Multiaddr>>),
    PeerId(OneshotSender<PeerId>),
}

async fn task(
    mut swarm: Swarm,
    mut event_stream: BehaviourEventStream,
    mut events_handler: Box<dyn RuntimeEvents>,
    topic: Topic,
    mut rx: Receiver<RuntimeAction>,
) {
    loop {
        futures::select_biased! {
            action = rx.next().fuse() => {
                let action = action.unwrap_or(RuntimeAction::Stop);

                match action {
                    RuntimeAction::Bootstrap => {
                        swarm.bootstrap();
                    }
                    RuntimeAction::Kbuckets(tx) => {
                        let kbuckets = swarm
                            .kademlia()
                            .kbuckets()
                            .map(|kbucket| kbucket.iter().map(|entry| entry.to_owned()).collect())
                            .collect();

                        tx.send(kbuckets).ok();
                    }
                    RuntimeAction::NetworkInfo(tx) => {
                        tx.send(Swarm::network_info(&swarm)).ok();
                    }
                    RuntimeAction::Stop => {
                        rx.close();
                        break;
                    }
                    RuntimeAction::Dial(address) => {
                        if let Err(e) = Swarm::dial_addr(&mut swarm, address.clone()) {
                            log::error!(
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
                            log::error!(
                                target: "locha-p2p",
                                "couldn't send message: {:?}",
                                e
                            );
                        }
                    }
                    RuntimeAction::ExternalAddresses(tx) => {
                        let addrs: Vec<Multiaddr> = Swarm::external_addresses(&swarm)
                            .map(|a| a.clone())
                            .collect();

                        tx.send(addrs).ok();
                    }
                    RuntimeAction::PeerId(tx) => {
                        tx.send(Swarm::local_peer_id(&swarm).clone()).ok();
                    }
                }
            },
            behaviour_event = event_stream.next().fuse() => {
                match behaviour_event.unwrap() {
                    BehaviourEvent::Message(_, _, msg) => {
                        events_handler.on_new_message(msg);
                    }
                }
            },
            ev = swarm.next_event().fuse() => {
                handle_event(&mut *events_handler, &ev).await;
            },
        }
    }
}

async fn handle_event<THandleErr: std::error::Error>(
    events_handler: &mut dyn RuntimeEvents,
    swarm_event: &SwarmEvent<(), THandleErr>,
) {
    match *swarm_event {
        SwarmEvent::Behaviour(_) => (),
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

//fn handle_behaviour_event(
//    swarm: &mut Swarm,
//    events_handler: &mut dyn RuntimeEvents,
//    event: &NetworkEvent,
//) {
//    match *event {
//        NetworkEvent::Gossipsub(ref gossip_ev) => {
//            if let GossipsubEvent::Message(ref _peer, ref _id, ref message) =
//                **gossip_ev
//            {
//                let contents = String::from_utf8_lossy(message.data.as_slice())
//                    .into_owned();
//                events_handler.on_new_message(contents);
//            }
//        }
//        NetworkEvent::Discovery(ref disc_ev) => match *disc_ev {
//            DiscoveryEvent::Discovered(ref peer) => {
//                log::info!(
//                    target: "locha-p2p",
//                    "found peer {}",
//                    peer,
//                );
//
//                for addr in swarm.addresses_of_peer(peer) {
//                    swarm.kademlia().add_address(peer, addr);
//                }
//
//                match Swarm::dial(swarm, peer) {
//                    Ok(()) => (),
//                    Err(e) => {
//                        warn!(
//                            target: "locha-p2p",
//                            "couldn't dial peer {}: {}",
//                            peer, e,
//                        );
//                    }
//                }
//            }
//            DiscoveryEvent::UnroutablePeer(_) => {}
//        },
//    }
//}
