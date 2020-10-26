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

//! # Runtime events
//!
//! These are the network events reported during the operation of the node. The
//! type must be `Send` because it will be used on the runtime task to report
//! the events.
//!
//! Aditionally the wrapper [`RuntimeEventsLogger`] around [`RuntimeEvents`] is
//! provided to log all events.
//!
//! The trait [`RuntimeEvents`] provides blanket implementation of all methods
//! so all of them don't have to be implemented.

use std::io;
use std::num::NonZeroU32;

use libp2p::core::connection::{ConnectedPoint, PendingConnectionError};
use libp2p::{Multiaddr, PeerId};

use log::{debug, info};

/// Chat service events
#[allow(unused_variables)]
pub trait RuntimeEvents: Send {
    /// New message received
    ///
    /// # Parameters
    ///
    /// - `peer_id`: The originator peer ID.
    /// - `message`: The message contents.
    fn on_new_message(&mut self, peer_id: &PeerId, message: String) {}

    /// Connection established to the given peer.
    fn on_connection_established(
        &mut self,
        peer: &PeerId,
        endpoint: &ConnectedPoint,
        num_established: NonZeroU32,
    ) {
    }

    /// Connection closed with the given peer.
    fn on_connection_closed(
        &mut self,
        peer: &PeerId,
        endpoint: &ConnectedPoint,
        num_established: u32,
        cause: Option<String>,
    ) {
    }

    /// New incoming connection
    fn on_incomming_connection(
        &mut self,
        local_addr: &Multiaddr,
        send_back_addr: &Multiaddr,
    ) {
    }

    /// Incoming connection failed
    fn on_incomming_connection_error(
        &mut self,
        local_addr: &Multiaddr,
        send_back_addr: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
    ) {
    }

    /// Peer on the given endpoint was banned.
    fn on_banned_peer(&mut self, peer: &PeerId, endpoint: &ConnectedPoint) {}

    /// Unreachable peer address
    fn on_unreachable_addr(
        &mut self,
        peer: &PeerId,
        address: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
        attempts_remaining: u32,
    ) {
    }

    /// Unknown peer is unreachable
    fn on_unknown_peer_unreachable_addr(
        &mut self,
        address: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
    ) {
    }

    /// New listening address
    ///
    /// This informs that the Chat Service is listening on the provided
    /// Multiaddress.
    fn on_new_listen_addr(&mut self, address: &Multiaddr) {}

    /// A listening address expired
    fn on_expired_listen_addr(&mut self, address: &Multiaddr) {}

    /// Listener closed.
    fn on_listener_closed(
        &mut self,
        addresses: &[Multiaddr],
        reason: &Result<(), io::Error>,
    ) {
    }

    /// An error ocurred on a listener.
    fn on_listener_error(&mut self, error: &io::Error) {}

    /// When dialing the given peer.
    fn on_dialing(&mut self, peer: &PeerId) {}
}

/// Small wrapper around [`RuntimeEvents`] that logs all events.
pub struct RuntimeEventsLogger<T: RuntimeEvents>(T);

impl<T> RuntimeEventsLogger<T>
where
    T: RuntimeEvents,
{
    /// Create a new [`RuntimeEventsLogger`]
    pub fn new(events_handler: T) -> Self {
        RuntimeEventsLogger(events_handler)
    }
}

impl<T> RuntimeEvents for RuntimeEventsLogger<T>
where
    T: RuntimeEvents,
{
    fn on_new_message(&mut self, peer_id: &PeerId, message: String) {
        debug!("new message {}", message);

        self.0.on_new_message(peer_id, message)
    }

    fn on_connection_established(
        &mut self,
        peer: &PeerId,
        endpoint: &ConnectedPoint,
        num_established: NonZeroU32,
    ) {
        match endpoint {
            ConnectedPoint::Dialer { address } => {
                debug!(
                    target: "locha-p2p",
                    "Outbound connection to peer {} on address {} succeed. \
                    Total number of established connections to peer are {}",
                    peer, address, num_established
                );
            }
            ConnectedPoint::Listener {
                local_addr,
                send_back_addr,
            } => {
                debug!(
                    target: "locha-p2p",
                    "Inbound connection to peer {} established on our address {} succeed. \
                    The stack of protocols for sending back to this peer are {}. \
                    Total number of established connections to this peer are {}",
                    peer, local_addr, send_back_addr, num_established
                );
            }
        };

        self.0
            .on_connection_established(peer, endpoint, num_established)
    }

    fn on_connection_closed(
        &mut self,
        peer: &PeerId,
        endpoint: &ConnectedPoint,
        num_established: u32,
        cause: Option<String>,
    ) {
        match endpoint {
            ConnectedPoint::Dialer { address } => match cause {
                Some(ref cause) => {
                    info!(
                        target: "locha-p2p",
                        "Outbound connection to peer {} on address {} failed. \
                        The cause of the close is {}. \
                        The number of remaining connections to peer {}",
                        peer, address, cause, num_established
                    );
                }
                None => {
                    info!(
                        target: "locha-p2p",
                        "Outbound connection to peer {} on address {} failed. \
                        The number of remaining connections to peer are {}",
                        peer, address, num_established
                    );
                }
            },
            ConnectedPoint::Listener {
                local_addr,
                send_back_addr,
            } => match cause {
                Some(ref cause) => {
                    info!(
                        target: "locha-p2p",
                        "Inbound connection from peer {} on our local address {} failed. \
                        The cause of the close is {}. \
                        The stack of protocols for sending back for this peer are {}. \
                        The number of remaining connections to peer are {}",
                        peer, local_addr, cause, send_back_addr, num_established
                    );
                }
                None => {
                    info!(
                        target: "locha-p2p",
                        "Inbound connection from peer {} on our local address {} failed. \
                        The stack of protocols for sending back for this peer are {}. \
                        The number of remaining connections to peer are {}",
                        peer, local_addr, send_back_addr, num_established
                    );
                }
            },
        };

        self.0
            .on_connection_closed(peer, endpoint, num_established, cause);
    }

    fn on_incomming_connection(
        &mut self,
        local_addr: &Multiaddr,
        send_back_addr: &Multiaddr,
    ) {
        debug!(
            target: "locha-p2p",
            "Incoming connection on {}, with protocols for sending back {}",
            local_addr,
            send_back_addr
        );

        self.0.on_incomming_connection(local_addr, send_back_addr);
    }

    fn on_incomming_connection_error(
        &mut self,
        local_addr: &Multiaddr,
        send_back_addr: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
    ) {
        debug!(
            target: "locha-p2p",
            "Incoming connection error on {}: {} \
            Protocols for sending back {}",
            local_addr, error, send_back_addr
        );

        self.0
            .on_incomming_connection_error(local_addr, send_back_addr, error);
    }

    fn on_banned_peer(&mut self, peer: &PeerId, endpoint: &ConnectedPoint) {
        debug!(
            target: "locha-p2p",
            "Peer {} is banned",
            peer
        );

        self.0.on_banned_peer(peer, endpoint);
    }

    fn on_unreachable_addr(
        &mut self,
        peer: &PeerId,
        address: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
        attempts_remaining: u32,
    ) {
        debug!(
            target: "locha-p2p",
            "Address {} for peer {} is unreachable. \
            Attempt failed with error {}. \
            Attempts remaining: {}",
            address, peer, error, attempts_remaining
        );

        self.0
            .on_unreachable_addr(peer, address, error, attempts_remaining);
    }

    fn on_unknown_peer_unreachable_addr(
        &mut self,
        address: &Multiaddr,
        error: &PendingConnectionError<io::Error>,
    ) {
        debug!(
            target: "locha-p2p",
            "Unknown peer address {} is unreachable. \
            Attempt failed with error {}",
            address, error
        );

        self.0.on_unknown_peer_unreachable_addr(address, error);
    }

    fn on_new_listen_addr(&mut self, address: &Multiaddr) {
        debug!(
            target: "locha-p2p",
            "Listening on new address {}",
            address
        );

        self.0.on_new_listen_addr(address);
    }

    fn on_expired_listen_addr(&mut self, address: &Multiaddr) {
        debug!(
            target: "locha-p2p",
            "Listening address {} expired",
            address
        );

        self.0.on_expired_listen_addr(address);
    }

    fn on_listener_closed(
        &mut self,
        addresses: &[Multiaddr],
        reason: &Result<(), io::Error>,
    ) {
        let mut affected = String::new();
        for address in addresses {
            affected.push_str(format!(", {}\n", address).as_str());
        }

        match *reason {
            Ok(_) => {
                debug!(
                    target: "locha-p2p",
                    "Listener closed successfully. \
                    Addresses closed: {}",
                    affected,
                );
            }
            Err(ref e) => {
                debug!(
                    target: "locha-p2p",
                    "Listener closed abruptly. Reason {}. \
                    Addresses affected: {}",
                    e, affected,
                );
            }
        };

        self.0.on_listener_closed(addresses, reason);
    }

    fn on_listener_error(&mut self, error: &io::Error) {
        debug!(target: "locha-p2p", "Listener error {}", error);

        self.0.on_listener_error(error);
    }

    fn on_dialing(&mut self, peer: &PeerId) {
        debug!(target: "locha-p2p", "Dialing peer {}", peer);

        self.0.on_dialing(peer);
    }
}
