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

use std::path::Path;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{load_yaml, App};

use futures::prelude::*;
use futures::select;

use async_std::io;
use async_std::sync::{channel, Sender};
use async_std::task;

use libp2p::multihash::{MultihashDigest, Sha2_256};

use serde_derive::{Deserialize, Serialize};

use locha_p2p::identity::Identity;
use locha_p2p::runtime::config::RuntimeConfig;
use locha_p2p::runtime::events::{RuntimeEvents, RuntimeEventsLogger};
use locha_p2p::runtime::Runtime;
use locha_p2p::{Multiaddr, PeerId};

use log::{info, trace};

mod arguments;
use arguments::Arguments;

struct EventsHandler {
    channel: Sender<Message>,
    echo: bool,
}

impl EventsHandler {
    fn send_echo(&self, message: String) {
        let mut message: Message = match serde_json::from_str(message.as_str())
        {
            Ok(m) => m,
            Err(_) => {
                trace!("not json message, --echo is enabled");
                return;
            }
        };

        let to_uid = message.from_uid.clone();
        let from_uid = message.to_uid.clone();

        message.to_uid = to_uid;
        message.from_uid = from_uid;

        message.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut hash_input = String::new();
        hash_input.push_str(message.from_uid.as_str());
        hash_input.push_str(message.to_uid.as_str());
        hash_input.push_str(message.msg.as_text());
        hash_input.push_str(message.timestamp.to_string().as_str());

        let mut hasher = Sha2_256::default();
        hasher.input(hash_input.as_bytes());
        message.msg_id = hex::encode(hasher.result().as_bytes());

        // TODO: why i can't use block_on inside a block_on :thinking:
        let handle = thread::spawn({
            let channel = self.channel.clone();

            move || task::block_on(async { channel.send(message).await })
        });

        handle.join().unwrap()
    }
}

impl RuntimeEvents for EventsHandler {
    fn on_new_message(&mut self, message: String) {
        info!("new message:\n{}", message);

        if self.echo {
            self.send_echo(message);
        }
    }

    fn on_new_listen_addr(&mut self, _: &Multiaddr) {}

    fn on_peer_discovered(&mut self, _: &PeerId, _: Vec<Multiaddr>) {}

    fn on_peer_unroutable(&mut self, _: &PeerId) {}
}

#[derive(Deserialize, Serialize)]
enum MessageContents {
    #[serde(rename = "text")]
    Text(String),
}

impl MessageContents {
    #[allow(unreachable_patterns)]
    pub fn as_text(&mut self) -> &str {
        match *self {
            MessageContents::Text(ref t) => t.as_str(),
            _ => panic!("Not text message content"),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Message {
    #[serde(rename = "fromUID")]
    pub from_uid: String,
    #[serde(rename = "toUID")]
    pub to_uid: String,
    pub msg: MessageContents,
    pub timestamp: u64,
    #[serde(rename = "type")]
    pub _type: u64,
    #[serde(rename = "msgID")]
    pub msg_id: String,
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let arguments = Arguments::from_matches(&matches);

    let mut runtime = Runtime::new();

    let identity = load_identity(&arguments.identity)
        .expect("couldn't load identity file");
    info!("our peer id: {}", identity.id());

    let config = RuntimeConfig {
        identity,
        channel_cap: 25,
        heartbeat_interval: 10,
        listen_addr: arguments.listen_addr,

        use_mdns: arguments.use_mdns,
        allow_ipv4_private: arguments.allow_ipv4_private,
        allow_ipv6_link_local: arguments.allow_ipv6_link_local,
        allow_ipv6_ula: arguments.allow_ipv6_ula,
    };

    let (sender, receiver) = channel::<Message>(10);
    let events_handler = EventsHandler {
        channel: sender,
        echo: arguments.echo,
    };

    runtime
        .start(config, Box::new(RuntimeEventsLogger::new(events_handler)))
        .expect("couldn't start chat service");

    // Reach out to another node if specified
    for to_dial in arguments.dials {
        runtime.dial(to_dial).expect("couldn't dial peer");
    }

    let input = io::stdin();
    let channel = receiver;

    task::block_on(async move {
        loop {
            let mut line = String::new();
            select! {
                _ = input.read_line(&mut line).fuse() => {
                    if line == "exit\n" || line == "exit\r\n" || line == "exit\r" {
                        break;
                    }

                    runtime
                        .send_message(line)
                        .expect("couldn't send message")
                }
                msg = channel.recv().fuse() => {
                    if let Ok(ref msg) = msg {
                        runtime.send_message(serde_json::to_string(msg).unwrap())
                            .expect("couldn't send message")
                    }
                }
            }
        }

        runtime.stop().expect("couldn't stop chat service")
    })
}

fn load_identity(file: &Path) -> std::io::Result<Identity> {
    Identity::from_file(file).or_else(|_| {
        let id = Identity::generate();
        id.to_file(file)?;
        Ok(id)
    })
}
