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

use libp2p::{Multiaddr, PeerId};

use log::{debug, info};

/// Chat service events
pub trait RuntimeEvents: Send {
    /// New message received
    fn on_new_message(&mut self, message: String);

    /// A peer has been discovered
    ///
    /// # Arguments
    ///
    /// - `peer` the now found peer.
    /// - `addrs` addresses for the peer, ordered by priority.
    fn on_peer_discovered(&mut self, peer: &PeerId, addrs: Vec<Multiaddr>);

    /// Discovery process determined that the given peer is unroutable.
    ///
    /// # Arguments
    ///
    /// - `peer` the unroutable peer.
    fn on_peer_unroutable(&mut self, peer: &PeerId);

    /// New listening address
    ///
    /// This informs that the Chat Service is listening on the provided
    /// Multiaddress.
    fn on_new_listen_addr(&mut self, address: &Multiaddr);
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
    fn on_new_message(&mut self, message: String) {
        debug!("new message {}", message);

        self.0.on_new_message(message)
    }

    fn on_peer_discovered(&mut self, peer: &PeerId, addrs: Vec<Multiaddr>) {
        debug!(
            target: "locha-p2p",
            "discovered peer {}. Addresses: {:?}",
            peer, addrs
        );

        self.0.on_peer_discovered(peer, addrs)
    }

    fn on_peer_unroutable(&mut self, peer: &PeerId) {
        debug!(
            target: "locha-p2p",
            "unroutable peer {}",
            peer
        );

        self.0.on_peer_unroutable(peer)
    }

    fn on_new_listen_addr(&mut self, address: &Multiaddr) {
        info!(
            target: "locha-p2p",
            "Listening on new address {}",
            address
        );

        self.0.on_new_listen_addr(address);
    }
}
