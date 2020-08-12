use crate::protocol;
use libp2p_swarm::{
    KeepAlive,
    NegotiatedSubstream,
    SubstreamProtocol,
    ProtocolsHandler,
    ProtocolsHandlerUpgrErr,
    ProtocolsHandlerEvent
};
use std::{
    error::Error,
    io,
    fmt,
    num::NonZeroU32,
    task::{Context, Poll},
    time::Duration
};
use std::collections::VecDeque;
use void::Void;

pub struct ChatHandler;

impl ChatHandler {
    pub fn new() -> ChatHandler {
        ChatHandler
    }
}

pub struct ChatEvent;

pub struct ChatSuccess;

#[derive(Debug)]
pub struct ChatFailure;

impl std::fmt::Display for ChatFailure {
    fn fmt(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChatFailure")
    }
}

impl std::error::Error for ChatFailure {
}

pub type ChatResult = Result<ChatSuccess, ChatFailure>;

impl ProtocolsHandler for ChatHandler {
    type InEvent = Void;
    type OutEvent = ChatResult;
    type Error = ChatFailure;
    type InboundProtocol = protocol::Chat;
    type OutboundProtocol = protocol::Chat;
    type OutboundOpenInfo = ();
}