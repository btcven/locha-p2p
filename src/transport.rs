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

use std::io;
use std::time::Duration;

use libp2p::core::muxing::StreamMuxerBox;
use libp2p::core::transport::Boxed;
use libp2p::core::upgrade::{SelectUpgrade, Version};
use libp2p::dns::DnsConfig;
use libp2p::mplex::MplexConfig;
use libp2p::noise;
use libp2p::tcp::TcpConfig;
use libp2p::websocket::WsConfig;
use libp2p::yamux;
use libp2p::Transport;

use libp2p::identity::Keypair;
use libp2p::PeerId;

/// Builds the `Transport` used in Locha P2P
///
/// # Arguments
///
/// `- keypair`: The keypair used to make a secure channel using
/// the Noise protocol.
///
/// # Example
///
/// ```rust
/// use locha_p2p::identity::Identity;
///
/// let id = Identity::generate();
/// let _transport = locha_p2p::build_transport(&id.keypair())
///     .expect("Couldn't create transport");
/// ```
pub fn build_transport(
    keypair: &Keypair,
) -> io::Result<Boxed<(PeerId, StreamMuxerBox)>> {
    // Create our low level TCP transport, and on top of it create a
    // WebSockets transport. They can be used both at the same time.
    let tcp = TcpConfig::new().nodelay(true);
    let dns = DnsConfig::new(tcp)?;
    let ws = WsConfig::new(dns.clone());
    let transport = dns.or_transport(ws);

    // Use the noise protocol to handle encryption and negotiation.
    // Also we use yamux and mplex to multiplex connections to peers.
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(keypair)
        .expect("Signing noise static DH keypair failed.");

    Ok(transport
        .upgrade(Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(SelectUpgrade::new(
            yamux::Config::default(),
            MplexConfig::default(),
        ))
        .timeout(Duration::from_secs(20))
        .boxed())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn build_transport_ok() {
        use libp2p::identity::Keypair;

        let keypair = Keypair::generate_secp256k1();
        build_transport(&keypair).expect("could not create transport!");
    }
}
