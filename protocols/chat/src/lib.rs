pub mod handler;
pub mod protocol;

use std::{task::Context, task::Poll};

use libp2p_core::{Multiaddr, PeerId, connection::ConnectionId};
use libp2p_swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters};

use void::Void;

use handler::{ChatEvent, ChatHandler, ChatResult};

/// Locha Mesh Chat protocol behaviour
pub struct Chat;

impl Chat {
    /// Create a new Chat.
    pub fn new() -> Chat { Chat }
}

impl NetworkBehaviour for Chat {
    type ProtocolsHandler = ChatHandler;
    type OutEvent = ChatEvent;

    fn new_handler(&mut self, _peer_id: &PeerId) -> Self::ProtocolsHandler {
        ChatHandler::new()
    }

    fn addresses_of_peer(&mut self, _peer_id: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, _: &PeerId) {}

    fn inject_disconnected(&mut self, _: &PeerId) {}

    fn inject_event(&mut self, _peer: PeerId, _: ConnectionId, result: ChatResult) {
        unimplemented!();
    }

    fn poll(&mut self, _: &mut Context<'_>, _: &mut impl PollParameters) -> Poll<NetworkBehaviourAction<Void, ChatEvent>> {
        unimplemented!();
    }
}