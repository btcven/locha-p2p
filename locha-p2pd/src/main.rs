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

#![recursion_limit = "256"]

use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{load_yaml, App};

use futures::prelude::*;
use futures::select;

use async_std::sync::{channel, Receiver, Sender};
use async_std::task;

use libp2p::multihash::{MultihashDigest, Sha2_256};
use libp2p::Multiaddr;

use serde_derive::{Deserialize, Serialize};

use locha_p2p::Identity;
use locha_p2p::{ChatService, ChatServiceConfig, ChatServiceEvents};

use termion::event::Key;

use parking_lot::RwLock;

use log::{info, trace};

mod arguments;
mod ui;
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

impl ChatServiceEvents for EventsHandler {
    fn on_new_message(&mut self, message: String) {
        info!("new message:\n{}", message);

        if self.echo {
            self.send_echo(message);
        }
    }

    fn on_new_listen_addr(&mut self, multiaddr: Multiaddr) {
        info!("new listen addr: {}", multiaddr)
    }
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
    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let arguments = Arguments::from_matches(&matches);

    if arguments.log_only {
        env_logger::Builder::from_env("LOCHA_P2PD_LOG")
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    let mut chat_service = ChatService::new();

    let identity = load_identity(&arguments.identity)
        .expect("couldn't load identity file");
    info!("our peer id: {}", identity.id());

    let config = ChatServiceConfig {
        identity,
        channel_cap: 25,
        heartbeat_interval: 10,
        listen_addr: arguments.listen_addr.clone(),
    };

    let (sender, receiver) = channel::<Message>(10);
    let events_handler = EventsHandler {
        channel: sender,
        echo: arguments.echo,
    };

    chat_service
        .start(config, Box::new(events_handler))
        .expect("couldn't start chat service");

    // Reach out to another node if specified
    for to_dial in arguments.dials.iter() {
        chat_service.dial(to_dial).expect("couldn't dial peer");
    }

    task::block_on(async move {
        if arguments.log_only {
            log_only_loop(chat_service, receiver).await
        } else {
            terminal_ui_loop(chat_service, receiver, arguments).await
        }
    })
}

async fn log_only_loop(
    chat_service: ChatService,
    channel: Receiver<Message>,
) -> ! {
    loop {
        let msg = channel.recv().await;
        if let Ok(ref msg) = msg {
            chat_service
                .send_message(serde_json::to_string(msg).unwrap())
                .expect("couldn't send message")
        }
    }
}

async fn terminal_ui_loop(
    mut chat_service: ChatService,
    channel: Receiver<Message>,
    arguments: Arguments,
) {
    let ui_data = Arc::new(RwLock::new(ui::Data::new()));
    let mut ui = ui::UserInterface::new(ui_data.clone()).unwrap();
    let events =
        ui::Events::new(Duration::from_millis(arguments.tui_tick_rate));

    loop {
        select! {
            msg = channel.recv().fuse() => {
                if let Ok(ref msg) = msg {
                    chat_service.send_message(serde_json::to_string(msg).unwrap())
                        .expect("couldn't send message")
                }
            }
            event = events.next().fuse() => {
                if event.is_err() {
                    break;
                }

                match event.unwrap() {
                    ui::Event::Tick => {
                        ui.tick().unwrap();
                    }
                    ui::Event::Input(Key::Char(c)) => {
                        if c == 'q' {
                            break;
                        } if c == '\n' {
                            let contents = ui_data.write().input_send();
                            chat_service.send_message(contents).unwrap();
                        } else {
                            ui_data.write().input_push(c);
                        }
                    }
                    ui::Event::Input(Key::Backspace) => {
                        ui_data.write().input_backspace();
                    }
                    _ => (),
                }
            }
        }
    }

    chat_service.stop().unwrap();
}

fn load_identity(file: &Path) -> std::io::Result<Identity> {
    Identity::from_file(file).or_else(|_| {
        let id = Identity::generate();
        id.to_file(file)?;
        Ok(id)
    })
}
