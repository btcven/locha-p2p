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

//! # Locha P2P
//!
//! This library contains the necessary code to set up a node that works
//! with the Locha P2P Chat.

#![recursion_limit = "1024"]
#![doc(html_logo_url = "https://locha.io/i/128.png")]
#![doc(html_favicon_url = "https://locha.io/i/128.png")]

pub mod discovery;
pub mod gossip;
pub mod identity;
pub mod network;
pub mod runtime;
pub mod sync;
pub mod upnp;

pub use libp2p::Multiaddr;
pub use libp2p::PeerId;

mod transport;
pub use transport::build_transport;

/// Locha P2P swarm
pub type Swarm = libp2p::Swarm<self::network::Network>;

/// Builds a swarm for Locha P2P
///
/// # Arguments
///
/// - `identity`: The node identity.
/// - `discovery_config`: Configuration for [`crate::discovery::DiscoveryBehaviour`].
///
/// # Errors
///
/// This function might return an error if transport creation
/// ([`build_transport`]) fails.
pub fn build_swarm(
    identity: &self::identity::Identity,
    discovery_config: self::discovery::DiscoveryConfig,
) -> Result<Swarm, std::io::Error> {
    let transport = build_transport(&identity.keypair())?;
    let discovery =
        self::discovery::DiscoveryBehaviour::with_config(discovery_config);
    let behaviour = self::network::Network::with_discovery(identity, discovery);

    Ok(Swarm::new(transport, behaviour, identity.id()))
}
