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

use libp2p::NetworkBehaviour;

use crate::discovery::{DiscoveryBehaviour, DiscoveryEvent};
use crate::gossip::{Gossipsub, GossipsubEvent, PublishError, Topic};

use crate::identity::Identity;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
#[behaviour(out_event = "NetworkEvent")]
pub struct Network {
    discovery: DiscoveryBehaviour,
    gossip: Gossipsub,
}

impl Network {
    pub fn with_discovery(
        identity: &Identity,
        discovery: DiscoveryBehaviour,
    ) -> Network {
        Network {
            discovery,
            gossip: crate::gossip::new(identity),
        }
    }

    pub fn subscribe(&mut self, topic: Topic) -> bool {
        self.gossip.subscribe(topic)
    }

    pub fn publish(
        &mut self,
        topic: &Topic,
        data: impl Into<Vec<u8>>,
    ) -> Result<(), PublishError> {
        self.gossip.publish(topic, data)
    }
}

/// Network behaviour event
pub enum NetworkEvent {
    Discovery(DiscoveryEvent),
    Gossipsub(Box<GossipsubEvent>),
}

impl From<DiscoveryEvent> for NetworkEvent {
    fn from(event: DiscoveryEvent) -> NetworkEvent {
        NetworkEvent::Discovery(event)
    }
}

impl From<GossipsubEvent> for NetworkEvent {
    fn from(event: GossipsubEvent) -> NetworkEvent {
        NetworkEvent::Gossipsub(Box::new(event))
    }
}
