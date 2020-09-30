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

use crate::{Multiaddr, PeerId};

pub mod behaviour;
pub(crate) mod pubsub;

pub use self::behaviour::{Behaviour, BehaviourEvent, BehaviourEventStream};

/// Locha P2P swarm
pub type Swarm = libp2p::Swarm<Behaviour>;

/// Builds a swarm for Locha P2P
///
/// # Arguments
///
/// - `identity`: The node identity.
/// - `upnp`: Wether to use UPnP.
/// - `mdns`: Wether to use mDNS local peer discovery.
/// - `event_chan_size`: The size of the events stream.
/// - `bootstrap_nodes`: A list of nodes for initial bootstrapping.
///
/// # Errors
///
/// This function might return an error if transport creation
/// ([`build_transport`]) fails.
///
/// # Example
///
/// ```rust
/// use locha_p2p::identity::Identity;
/// use locha_p2p::p2p::build_swarm;
///
/// let id = Identity::generate();
/// let _swarm = build_swarm(
///     &id,
///     false,
///     false,
///     10,
///     std::iter::empty()
/// ).unwrap();
/// ```
pub fn build_swarm<'iter>(
    identity: &crate::identity::Identity,
    upnp: bool,
    mdns: bool,
    events_chan_size: usize,
    bootstrap_nodes: impl Iterator<Item = &'iter (PeerId, Multiaddr)>,
) -> Result<(Swarm, BehaviourEventStream), std::io::Error> {
    let transport = crate::transport::build_transport(&identity.keypair())?;
    let (behaviour, evt_stream) =
        Behaviour::new(identity, upnp, mdns, events_chan_size, bootstrap_nodes);

    Ok((Swarm::new(transport, behaviour, identity.id()), evt_stream))
}

#[test]
fn test_build_swarm() {
    use crate::identity::Identity;

    let id = Identity::generate();
    build_swarm(&id, false, false, 1, std::iter::empty()).unwrap();
}
