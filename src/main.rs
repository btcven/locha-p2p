#![recursion_limit = "256"]

use async_trait::async_trait;
use futures::{prelude::*, select};
use libp2p::{
    core::upgrade::{read_one, write_one},
    identity,
    ping::{Ping, PingConfig},
    request_response::{
        ProtocolName, ProtocolSupport, RequestResponse, RequestResponseCodec,
        RequestResponseConfig, RequestResponseEvent, RequestResponseMessage,
    },
    swarm::{Swarm, SwarmEvent},
    PeerId,
};
use std::{io, iter};

#[derive(Debug, Clone)]
pub struct Chat;

impl ProtocolName for Chat {
    fn protocol_name(&self) -> &[u8] {
        b"/locha-p2p/chat/1.0.0"
    }
}

#[derive(Debug, Clone)]
pub struct ChatCodec;

#[derive(Debug, Clone)]
pub struct Message {
    content: String,
}

#[derive(Debug, Clone)]
pub struct MessageConfirmation;

#[async_trait]
impl RequestResponseCodec for ChatCodec {
    type Protocol = Chat;
    type Request = Message;
    type Response = MessageConfirmation;

    async fn read_request<T>(&mut self, _: &Chat, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = read_one(io, 1024)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .await?;

        let content =
            String::from_utf8(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e));
        Ok(Message { content })
    }

    async fn read_response<T>(&mut self, _: &Chat, io: &mut T) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let magic = read_one(io, 8)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .await?;

        if magic != b"\xDE\xAD\xBE\xEF\xCA\xFE\xCA\xFE" {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid magic value",
            ))
        } else {
            Ok(MessageConfirmation)
        }
    }

    async fn write_request<T>(&mut self, _: &Chat, io: &mut T, message: Message) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_one(io, message.content.as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .await
    }

    async fn write_response<T>(
        &mut self,
        _: &Chat,
        io: &mut T,
        _: MessageConfirmation,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_one(io, b"\xDE\xAD\xBE\xEF\xCA\xFE\xCA\xFE")
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            .await
    }
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Create a random PeerId.
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);

    // Create a transport.
    let transport = libp2p::build_development_transport(id_keys).unwrap();

    let cfg = RequestResponseConfig::default();
    let protocols = iter::once((Chat, ProtocolSupport::Full));
    let behaviour = RequestResponse::new(ChatCodec, protocols, cfg);

    // Create a Swarm that establishes connections through the given transport
    // and applies the ping behaviour on each connection.
    let mut swarm = Swarm::new(transport, behaviour, peer_id);

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote = addr.parse().unwrap();
        Swarm::dial_addr(&mut swarm, remote).unwrap();
        println!("Dialed {}", addr)
    }

    // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
    Swarm::listen_on(&mut swarm, "/ip6/::/tcp/0".parse().unwrap()).unwrap();

    for addr in Swarm::listeners(&swarm) {
        println!("Listening on {}", addr);
    }

    let mut line = String::new();
    let stdin = async_std::io::stdin();
    let mut stdout = async_std::io::stdout();
    stdout.write_all(b"> ").await?;
    // Main loop
    'outer: loop {
        select! {
            // Poll for next libp2p event
            event = swarm.next_event().fuse() => {
                match event {
                    SwarmEvent::Behaviour(b) => {
                        match b {
                            RequestResponseEvent::Message { peer, message } => {}
                            _ => unimplemented!(),
                        }
                    }
                    SwarmEvent::NewListenAddr(multiaddr) => {
                        stdout.write_all(format!("Listening on: {}\n", multiaddr).as_bytes()).await?;
                        stdout.flush().await?;
                    }
                    e => println!("event: {:?}", e),
                }
            }
            // Read a line
            count = stdin.read_line(&mut line).fuse() => {
                if let Err(e) = count {
                    return Err(e);
                }
                if line == "exit\n" || line == "exit\r\n" || line == "exit\r" {
                    stdout.write_all(b"Exiting...\n").await?;
                    break 'outer;
                } else {
                    for
                    swarm.send_request();
                }
                stdout.write_all(b"> ").await?;
                stdout.flush().await?;
                line.clear();
            }
        }
    }

    Ok(())
}
