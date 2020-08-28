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

//! # Gosssipsub behaviour

use std::time::Duration;

use libp2p::gossipsub::GossipsubConfig;
use libp2p::gossipsub::GossipsubConfigBuilder;
use libp2p::gossipsub::{GossipsubMessage, MessageAuthenticity, MessageId};

use crate::identity::Identity;

pub use libp2p::gossipsub::error::PublishError;
pub use libp2p::gossipsub::{Gossipsub, GossipsubEvent, Topic};

/// Gossipsub protocol name for Locha P2P Chat
pub const CHAT_SERVICE_GOSSIP_PROTCOL_ID: &[u8] = b"/locha/gossip/1.0.0";

/// New gossipsub behaviour
pub fn new(identity: &Identity) -> Gossipsub {
    Gossipsub::new(
        MessageAuthenticity::Signed(identity.keypair()),
        gossipsub_config(),
    )
}

/// Generate Gossipsub configuration
fn gossipsub_config() -> GossipsubConfig {
    GossipsubConfigBuilder::new()
        .protocol_id(CHAT_SERVICE_GOSSIP_PROTCOL_ID)
        .heartbeat_interval(Duration::from_secs(5))
        .message_id_fn(gossipsub_message_id)
        .build()
}

/// Calculate MessageId for a Gossipsub message.
///
/// We use Keccak-384 because it's fast, lightweight, with not known attacks
/// and very little probabilities of collisions.
fn gossipsub_message_id(message: &GossipsubMessage) -> MessageId {
    use libp2p::multihash::{Keccak384, MultihashDigest};

    let mut hasher = Keccak384::default();
    hasher.input(message.data.as_slice());

    MessageId::from(hasher.result().into_bytes())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn gossipsub_config_ok() {
        // GossipsubConfigBuilder can generate panics on invalid values
        gossipsub_config();
    }

    #[test]
    fn gossipsub_message_id_works() {
        use libp2p::gossipsub::GossipsubMessage;

        let message = GossipsubMessage {
            source: None,
            data: vec![0xca, 0xfe, 0xca, 0xfe],
            sequence_number: None,
            topics: vec![],
            signature: None,
            key: None,
            validated: false,
        };

        let id = gossipsub_message_id(&message);
        assert_eq!(id.to_string(), "1c30927f61ecac8caeccc3db58f0b48d6468a4bbdfeb178f9b9ce97147abeed66b39d361996173fed138c7cee4e08ac18fae".to_string());
    }
}
